import { WasmModule } from '../wasm/wasm'
import { EventBus } from './bus'
import { highlight } from './highlight'
import { Input } from './input'

export interface Position {
    absoluteIndex: number,
    lineIndex: number,
    charIndex: number
}

export interface Caret {
    from: Position
    to?: Position
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

    let absoluteIndex = 0
    let lineIndex = 0

    for (const line of document.childNodes) {
        if (line.contains(node) || line === node) {
            const length = positionInNode(range, line, side)

            return {
                absoluteIndex: absoluteIndex + length,
                charIndex: length,
                lineIndex: lineIndex
            }
        }

        absoluteIndex += elementLength(line)
        lineIndex += 1
    }

    return {
        absoluteIndex: 0,
        lineIndex: 0,
        charIndex: 0
    }
}

export class Editor {
    element: HTMLDivElement
    module: WasmModule
    bus: EventBus

    constructor(parent: HTMLDivElement, module: WasmModule, bus: EventBus) {
        this.element = this.#createElement(parent) 
        this.module = module
        this.bus = bus

        this.#setup()
    }

    #setup() {
        this.element.addEventListener('input', () => {
            this.#highlight()
        })
    }

    #createElement(parent: HTMLDivElement): HTMLDivElement {
        parent.innerHTML = `<div class="code-editor" contenteditable="true" spellcheck="false" />`
        return parent.querySelector<HTMLDivElement>('div.code-editor')!
    }

    #highlight() {
        const caret = this.getCaret()
        const content = this.getContent()

        this.setContent(...content)
        if (caret) this.setCaret(caret)
    }

    focus() {
        this.element.focus()
    }

    getCaret(): Caret | null {
        const selection = document.getSelection()

        if (!selection) {
            return null
        }
    
        let range = selection.getRangeAt(0)
    
        if (range.startOffset === range.endOffset && range.startContainer === range.endContainer) {
            return {
                from: positionInElement(selection, this.element, 'start')
            }
        }
    
        return {
            from: positionInElement(selection, this.element, 'start'),
            to: positionInElement(selection, this.element, 'end')
        }
    }

    setCaret(caret: Caret) {
        const selection = window.getSelection();

        if (!selection) {
            return
        }

        const range = document.createRange();

        let { lineIndex, charIndex } = caret.from 
        let node = this.element.childNodes[lineIndex]
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

        range.setStart(node, charIndex)
        range.setEnd(node, charIndex)

        selection?.removeAllRanges()
        selection?.addRange(range)
    }

    selectCurrentLine() {
        const selection = document.getSelection()

        if (!selection) {
            return
        }

        const range = document.createRange()
        const { lineIndex } = this.getCaret()!.from

        const line = this.element.childNodes[lineIndex]

        range.setStart(line, 0)
        range.setEnd(line, 1)

        selection.removeAllRanges()
        selection.addRange(range)
    }

    getContent(): string[] {
        const lines = Array.from(this.element.childNodes)
        return lines.map(node => node.textContent ?? "")
    }

    setContent(...content: string[]) {
        const input = content.join('\n')
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
    
        this.element.innerHTML = html

        this.bus.publish('contentChanged', {
            editor: this,
            content
        })
    }
}

