import { WasmModule } from '../wasm/wasm'
import { EventBus } from './bus'
import { highlight } from './highlight'
import { Undo } from './undo'

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

export class Editor {
    gutter: HTMLDivElement
    input: HTMLDivElement
    module: WasmModule
    bus: EventBus
    state: Undo

    constructor(parent: HTMLDivElement, module: WasmModule, bus: EventBus) {
        const [gutter, input] = this.createElement(parent) 
        this.gutter = gutter
        this.input = input
        this.module = module
        this.bus = bus
        this.state = new Undo()

        this.input.addEventListener('input', () => {
            this.highlight()
        })

        this.input.addEventListener('copy', (e) => {
            const caret = this.getCaret()
            if (!caret) return
            if (caret.to) return
            
            e.preventDefault()
            this.writeCurrentLineToClipboard(caret)
        })

        this.input.addEventListener('cut', (e) => {
            const caret = this.getCaret()
            if (!caret) return
            if (caret.to) return
            
            e.preventDefault()
            this.writeCurrentLineToClipboard(caret)
            this.removeLineByIndex(caret.from.lineIndex)
            this.setCaret(toStartOfLine(caret))
        })

        this.input.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key == "z") {
                e.preventDefault()
                this.undo()
            } else if (e.ctrlKey && e.key == "y") {
                e.preventDefault()
                this.redo()
            }

            const caret = this.getCaret()

            if (e.key == "Enter" || e.key == " ") {
                if (!caret) {
                    return
                }

                this.state.push({
                    caret,
                    input_html: this.input.innerHTML,
                    gutter_html: this.gutter.innerHTML
                });
            }

            if (e.key == "Tab") {
                if (!caret) {
                    return
                }

                e.preventDefault()
                
                this.state.push({
                    caret,
                    input_html: this.input.innerHTML,
                    gutter_html: this.gutter.innerHTML
                });

                const offset = caret.from.charIndex % 2;
                const whitespace = " ".repeat(offset === 0 ? 2 : offset)

                this.insert(caret.from.lineIndex, caret.from.charIndex, whitespace)

                this.setCaret({
                    from: {
                        lineIndex: caret.from.lineIndex,
                        charIndex: caret.from.charIndex + whitespace.length
                    }
                })

            }
        })
    }

    private createElement(parent: HTMLDivElement): [HTMLDivElement, HTMLDivElement] {
        parent.innerHTML = 
        `<div class="code-editor">
            <div class="gutter"><span>1</span></div>
            <div class="input" contenteditable="true" spellcheck="false"></div>
        </div>`;

        const gutter = parent.querySelector<HTMLDivElement>('div.code-editor div.gutter')!
        const input = parent.querySelector<HTMLDivElement>('div.code-editor div.input')!

        return [gutter, input]
    }

    private highlight() {
        const caret = this.getCaret()
        const content = this.getContent()

        this.setContent(...content)
        if (caret) this.setCaret(caret)
    }

    insert(line: number, col: number, value: string) {
        const content = this.getContent()
        const before = content[line].slice(0, col)
        const after = content[line].slice(col)

        content[line] = before + value + after

        this.setContent(...content)
    }

    focus() {
        this.input.focus()
    }

    getCaret(): Caret | null {
        const selection = document.getSelection()

        if (!selection) {
            return null
        }
    
        try {
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
        } catch {
            return null
        }
    }

    setCaret(caret: Caret | null) {
        if (!caret) {
            this.focus()
            return
        }

        const selection = window.getSelection();

        if (!selection) {
            this.focus()
            return
        }

        const range = document.createRange();

        let { lineIndex, charIndex } = caret.from 
        let node = this.input.childNodes[lineIndex]

        if (!node) {
            this.focus()
            return
        }

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

    undo() {
        const state = this.state.undo()
        if (!state) return

        console.log(state)

        this.input.innerHTML = state.input_html
        this.gutter.innerHTML = state.gutter_html

        if (state.caret) this.setCaret(state.caret)
    }

    redo() {
        const state = this.state.redo()
        if (!state) return

        this.input.innerHTML = state.input_html
        this.gutter.innerHTML = state.gutter_html

        if (state.caret) this.setCaret(state.caret)
    }

    getContent(): string[] {
        const lines = Array.from(this.input.childNodes)
        return lines.map(node => node.textContent ?? "")
    }

    setContent(...content: string[]) {
        const input = content.join('\n')
        const lines = highlight(this.module, input)

        let gutter_html = ""
        let input_html = ""
    
        lines.forEach(({ fragments }, lineIndex) => {
            gutter_html += `<span>${lineIndex + 1}</span>`

            if (fragments.length === 0) {
                input_html += `<div class="ln"><br></div>`
                return
            }

            input_html += `<div class="ln">`
    
            fragments.forEach(({ value, kind, error, hint }) => {
                if (kind || error || hint) {
                    let classes = [kind, error].filter(Boolean).join(" ")
                    let title = Boolean(hint) ? hint : ""
                    input_html += `<span class="${classes}" title="${title}">${value}</span>`
                } else {
                    input_html += value
                }
            })
    
            input_html += `</div>`
        })

        // This is a case where a single fragment containing an empty string as value
        // is returned. We do not want this empty div element to be in the contenteditable
        // because it will stay trailing behind the normal input.
        if (input_html === `<div class="ln"></div>`) {
            input_html = ""
        }
    
        this.gutter.innerHTML = gutter_html
        this.input.innerHTML = input_html
  
        this.bus.publish('contentChanged', {
            editor: this,
            content
        })
    }
}

