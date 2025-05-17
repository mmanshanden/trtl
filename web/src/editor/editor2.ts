import { WasmModule } from "../wasm/wasm"
import { highlight } from "./highlight"


export interface Position {
    lineIndex: number,
    charIndex: number
}

export interface Caret {
    from: Position
    to?: Position
}

interface State {
    lines: string[]
}

interface Input {
    inputType: 'insertText' | 'deleteContentBackward'
    data: string
}

export class Editor2 {
    gutter: HTMLDivElement
    input: HTMLDivElement
    module: WasmModule
    state: State

    constructor(parent: HTMLDivElement, module: WasmModule) {
        const [gutter, input] = this.createElement(parent) 
        this.gutter = gutter
        this.input = input
        this.module = module

        this.state = {
            lines: [""]
        }

        this.input.addEventListener('beforeinput', (e) => {
            e.preventDefault()
            this.handleInputEvent(e as InputEvent)
        })

        this.input.addEventListener('input', (e) => {
            e.preventDefault()
        })

        this.render()
        
        this.focus()
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

    private render() {
        const input = this.state.lines.join('\n')
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
    
        this.gutter.innerHTML = gutter_html
        this.input.innerHTML = input_html
    }

    private handleInputEvent(e: InputEvent) {
        const caret = this.getCaret()

        if (caret == null) {
            throw new Error("INPUT_WITHOUT_CARET")
        }

        console.log("input", e)

        switch (e.inputType) {
            case "deleteContentBackward":
                break
            case "insertParagraph":
            case "insertLineBreak":
                this.insertLine()
                this.render()

                this.setCaret({
                    from: {
                        charIndex: 0,
                        lineIndex: caret.from.lineIndex + 1,
                    }
                })

                break
            case "insertText":
                this.insertText(e.data!)
                this.render()
                this.setCaret({
                    from: {
                        charIndex: caret.from.charIndex + 1,
                        lineIndex: caret.from.lineIndex
                    }
                })

                break
        }

    }

    private getCaretLineIndex(container: Node): number {
        if (container === this.input) {
            return 0;
        }

        let lineIndex = 0

        for (const line of this.input.childNodes) {
            if (line === container || line.contains(container)) {
                return lineIndex
            }

            lineIndex += 1;
        }

        throw new Error("CARRET_OUT_OF_RANGE")
    }

    private getCaretCharIndex(container: Node, offset: number): number {
        while (container != this.input && container.parentElement != this.input) {
            if (container.previousSibling != null) {
                container = container.previousSibling
                offset += container.textContent ? container.textContent.length : 0
                continue
            }

            container = container.parentElement!
        }

        return offset
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
                    from: {
                        lineIndex: this.getCaretLineIndex(range.startContainer),
                        charIndex: this.getCaretCharIndex(range.startContainer, range.startOffset)   
                    }
                }
            }
        
            return {
                from: {
                    lineIndex: this.getCaretLineIndex(range.startContainer),
                    charIndex: this.getCaretCharIndex(range.startContainer, range.startOffset)
                },
                to: {
                    lineIndex: this.getCaretLineIndex(range.endContainer),
                    charIndex: this.getCaretCharIndex(range.endContainer, range.endOffset)
                }
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

        const elementLength = (node: ChildNode): number => {
           return node.textContent?.length ?? 0
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

    selectedText(): string[] {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")
        if (!caret.to) return []

        const { lineIndex: fromLine, charIndex: fromChar } = caret.from
        const { lineIndex: toLine, charIndex: toChar } = caret.to

        if (fromLine === toLine) {
            return [this.state.lines[fromLine].slice(fromChar, toChar)]
        }

        return [
            this.state.lines[fromLine].slice(fromChar),
            ...this.state.lines.slice(fromLine + 1, toLine - 1),
            this.state.lines[toLine].slice(0, toChar)
        ]
    }

    insertLine() {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")

        const { lineIndex, charIndex } = caret.from
        
        
        this.state.lines = [
            ...this.state.lines.slice(0, lineIndex),
            this.state.lines[lineIndex].slice(0, charIndex),
            this.state.lines[lineIndex].slice(charIndex),
            ...this.state.lines.slice(lineIndex + 1)
        ]
    }

    insertText(text: string) {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")
        
        const { lineIndex, charIndex }  = caret.from

        const before = this.state.lines[lineIndex].slice(0, charIndex) 
        const after = this.state.lines[lineIndex].slice(charIndex)

        this.state.lines[lineIndex] = before + text + after
    }
}