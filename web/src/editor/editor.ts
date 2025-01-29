import { WasmModule } from '../wasm/wasm'
import { EventBus } from './bus'
import { highlight } from './highlight'

export interface Position {
    lineIndex: number,
    charIndex: number
}

export interface Caret {
    from: Position
    to?: Position
}

const toStartOfLine = (caret: Caret): Caret => {
    return {
        from: {
            lineIndex: caret.from.lineIndex,
            charIndex: 0
        }
    }
}

const elementLength = (node: ChildNode): number => {
    return node.textContent?.length ?? 0
}

const positionInNode = (range: Range, node: ChildNode, side: 'start' | 'end'): number => {
    const rel = range.cloneRange()

    rel.selectNodeContents(node)

    if (side === 'start') {
        rel.setEnd(range.startContainer, range.startOffset) 
    } else {
        rel.setEnd(range.endContainer, range.endOffset)
    }

    return rel.toString().length
}

const positionInElement = (selection: Selection, document: HTMLDivElement, side: 'start' | 'end'): Position => {
    const range = selection.getRangeAt(0)
    const node = side === 'start' ? range.startContainer : range.endContainer

    let lineIndex = 0

    for (const line of document.childNodes) {
        if (line.contains(node) || line === node) {
            const length = positionInNode(range, line, side)

            return {
                charIndex: length,
                lineIndex: lineIndex
            }
        }

        lineIndex += 1
    }

    return {
        lineIndex: 0,
        charIndex: 0
    }
}

type Content = string[]

interface UndoStack {
    stack: string[]
    pointer: number
}

export class Editor {
    mirror: HTMLDivElement
    input: HTMLDivElement
    module: WasmModule
    bus: EventBus
    undoStack: UndoStack = {
        stack: [],
        pointer: 0
    }

    constructor(parent: HTMLDivElement, module: WasmModule, bus: EventBus) {
        const [mirror, input] = this.createElement(parent) 
        this.mirror = mirror
        this.input = input
        this.module = module
        this.bus = bus

        this.input.addEventListener('input', (e) => {
            this.highlight()
        })

        this.input.addEventListener('copy', (e) => {
            const caret = this.getCaret()
            if (!caret) return
            if (caret.to) return

            this.selectCurrentLine()
        })

        this.input.addEventListener('cut', (e) => {
            const caret = this.getCaret()
            if (!caret) return
            if (caret.to) return
            
            this.selectCurrentLine()
        })
    }

    private createElement(parent: HTMLDivElement): [HTMLDivElement, HTMLDivElement] {
        parent.innerHTML = `
            <div class="code-editor">
                <div class="input" contenteditable="true" spellcheck="false"></div>
                <div class="mirror"><div class="ln"></div></div>
            </div>`
        const mirror = parent.querySelector<HTMLDivElement>('div.mirror')!
        const input = parent.querySelector<HTMLDivElement>('div.input')!
        return [mirror, input]
    }

    private highlight() {
        const input = this.getContent().join('\n')
        const lines = highlight(this.module, input)
        let html = ""
    
        lines.forEach(({ fragments }, lineIndex) => {
            html += `<div class="ln">`
    
            fragments.forEach(({ value, kind, error, hint }) => {
                if (value == "") {
                    html += '<br>'
                } else if (kind || error || hint) {
                    let classes = [kind, error].filter(Boolean).join(" ")
                    let title = Boolean(hint) ? hint : ""
                    html += `<span class="${classes}" title="${title}">${value}</span>`
                } else {
                    html += value
                }
            })
    
            if (lineIndex === lines.length - 1) {
                html += '<br>'
            }
    
            html += `</div>`
        })
    
        this.mirror.innerHTML = html
    }

    focus() {
        this.input.focus()
    }

    getCaret(): Caret | null {
        const selection = document.getSelection()

        if (!selection) {
            return null
        }
    
        let range = selection.getRangeAt(0)
    
        if (range.startOffset === range.endOffset && range.startContainer === range.endContainer) {
            return {
                from: positionInElement(selection, this.input, 'start')
            }
        }
    
        return {
            from: positionInElement(selection, this.input, 'start'),
            to: positionInElement(selection, this.input, 'end')
        }
    }

    setCaret(caret: Caret) {
        const selection = window.getSelection();

        if (!selection) {
            return
        }

        const range = document.createRange();

        let { lineIndex, charIndex } = caret.from 
        let node = this.input.childNodes[lineIndex]
        let stack = [...node.childNodes]

        while (stack.length > 0) {
            node = stack.shift()!

            if (!node) {
                continue
            }

            if (node.nodeType === Node.ELEMENT_NODE) {
                stack.unshift(...node.childNodes)
                continue
            }

            const length = elementLength(node)

            if (charIndex <= length) {
                break
            }

            charIndex = charIndex - length
        }

        range.setStart(node, Math.min(elementLength(node), charIndex))
        range.setEnd(node, Math.min(elementLength(node), charIndex))

        selection?.removeAllRanges()
        selection?.addRange(range)
    }

    writeCurrentLineToClipboard(caret: Caret) {
        const content = this.getContent()
        const line = content[caret.from.lineIndex]
        navigator.clipboard.writeText(line)
    }

    removeLineByIndex(index: number) {
        const content = this.getContent()
        content.splice(index, 1)
        this.setContent(...content)
    }

    selectCurrentLine() {
        const selection = document.getSelection()

        if (!selection) {
            return
        }

        const range = document.createRange()
        const { lineIndex } = this.getCaret()!.from

        const line = this.input.childNodes[lineIndex]

        range.setStart(line, 0)
        range.setEnd(line, line.childNodes.length)

        selection.removeAllRanges()
        selection.addRange(range)
    }

    setContent(...content: Content) {
        // this.input.innerHTML = ""
        // content.forEach(line => {
        //     const div = document.createElement('div')
        //     div.textContent = line
        //     div.classList.add('ln')
        //     this.input.appendChild(div)
        // })

        // this.highlight()
    }

    getContent(): Content {
        const lines = Array.from(this.input.childNodes)
        return lines.map(line => line.textContent ?? "")
    }
}

