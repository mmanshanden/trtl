import { WasmModule } from "../wasm/wasm"
import { highlight } from "./highlight"
import { deleteRange, insert, mergeLines, removeChars } from "./util"


const WORD_BOUNDARY = /[\s-\+\*\/\\\(\)\.\[\]\{\}\;\=\:\<\>\?\%\^\!\@\#\$\&\`\'\"]/

export interface Position {
    lineIndex: number,
    charIndex: number
}

export interface Caret {
    from: Position
    to?: Position
}


export class Editor extends EventTarget {
    gutter: HTMLDivElement
    input: HTMLDivElement
    module: WasmModule
    private content: string[]
    private caret: Caret | null

    constructor(parent: HTMLDivElement, module: WasmModule) {
        super()

        const [gutter, input] = this.createElement(parent) 
        this.gutter = gutter
        this.input = input
        this.module = module
        this.content = [""]
        this.caret = null

        this.input.addEventListener('beforeinput', (e) => {
            e.preventDefault()

            const event = e as InputEvent
            if (event.inputType === 'insertCompositionText') return
            if (event.isComposing) return


            this.caret = this.handleInputEvent(e as InputEvent)
            this.setCaret(this.caret)

        })

        this.input.addEventListener('input', (e) => {
            e.preventDefault()
            
            const event = e as InputEvent
            if (event.isComposing) return
            if (!event.isTrusted) return

            this.render()
            this.setCaret(this.caret)

        })

         this.input.addEventListener('compositionend', (e) => {
             e.preventDefault()

            this.caret = this.insertText(e.data)
            this.setCaret(this.caret)
        })

        this.input.addEventListener('keydown', (e) => {
            // qnd tab insert
            if (e.key === 'Tab') {
                const idx = this.getCaret()?.from.charIndex ?? 0
                const tab = idx % 4 == 0 ? "    " : " ".repeat(idx % 4)
                this.insertText(tab)
                e.preventDefault()
            }
        })

        this.render()
        this.focus()
    }

    private createElement(parent: HTMLDivElement): [HTMLDivElement, HTMLDivElement] {
        parent.innerHTML = 
            `<div class="code-editor">
                <div class="gutter"><span>1</span></div>
                <div class="input" contenteditable="true" autocapitalize="off" autocomplete="off" spellcheck="false" autocorrect="off"></div>
            </div>`;

        const gutter = parent.querySelector<HTMLDivElement>('div.code-editor div.gutter')!
        const input = parent.querySelector<HTMLDivElement>('div.code-editor div.input')!

        return [gutter, input]
    }

    // Renders the internal state to the DOM.
    //
    // Invariant: the node referenced by `this.input` contains one `div.ln` node for
    // for every line of the rendered state.
    private render() {
        const input = this.getContent()
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
                    input_html += `<span>${value}</span>`
                }
            })

            if (fragments.length === 0) {
                input_html += "<br />"
            }
    
            input_html += `</div>`
        })
    
        this.gutter.innerHTML = gutter_html
        this.input.innerHTML = input_html

        this.dispatchEvent(new Event("render"))
    }

    private handleInputEvent(e: InputEvent): Caret | null {
        switch (e.inputType) {
            case "deleteContentForward":
                return this.deleteForward()
            case "deleteByCut":
            case "deleteContentBackward":
                return this.deleteBackward()
            case "deleteWordBackward":
                return this.deleteBackward(true)
            case "insertParagraph":
            case "insertLineBreak":
                return this.insertLine()
            case "insertCompositionText":
            case "insertText":
                return this.insertText(e.data!)
            case "insertFromPaste":
                const data = e.dataTransfer ? e.dataTransfer.getData("text/plain") : ""
                return this.insertText(data)
        }

        return null
    }

    private getCaretLineIndex(container: Node, offset: number): number {
        if (container === this.input) {
            return offset
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
        // Walk back until the `div.ln` node is reached, which, as per the invariant,
        // is the node for which the parent is `this.input`

        while (container != this.input && container.parentElement != this.input) {
            if (container.previousSibling == null) {
                container = container.parentElement!
                continue
            }

            container = container.previousSibling
            offset += container.textContent ? container.textContent.length : 0
        }

        return offset
    }

    private startOfWord(position: Position): Position {
        const line = this.content[position.lineIndex]
        const char = line.charAt(position.charIndex - 1)

        const isBoundary = /\s/.test(char) ? /\S/ : WORD_BOUNDARY

        for (let i = position.charIndex - 1; i >= 0; i--) {
            if (isBoundary.test(line[i])) {
                return {
                    charIndex: i + 1 === position.charIndex ? i : i + 1,
                    lineIndex: position.lineIndex
                }
            }
        }

        return {
            charIndex: 0,
            lineIndex: position.lineIndex
        }
    }

    focus() {
        this.input.focus()
    }

    setContent(content: string) {
        this.content = content.split("\n")

        this.render()
        this.setCaret({
            from: {
                charIndex: 0,
                lineIndex: 0
            }
        })
    }

    getContent(): string {
        return this.content.join("\n")
    }

    getCaret(): Caret | null {
        const selection = document.getSelection()

        if (!selection) {
            return null
        }
    
        try {
            let range = selection.getRangeAt(0)
            
            // No "selection", just a caret at a position
            if (range.startOffset === range.endOffset && range.startContainer === range.endContainer) {
                return {
                    from: {
                        lineIndex: this.getCaretLineIndex(range.startContainer, range.startOffset),
                        charIndex: this.getCaretCharIndex(range.startContainer, range.startOffset)   
                    }
                }
            }

            // Browser selected everything
            if (range.endContainer === this.input) {
                return {
                    from: {
                        lineIndex: this.getCaretLineIndex(range.startContainer, range.startOffset),
                        charIndex: this.getCaretCharIndex(range.startContainer, range.startOffset)
                    },
                    to: {
                        lineIndex: this.content.length,
                        charIndex: this.content[this.content.length - 1].length
                    }
                }
            }
        
            // Actual selection
            return {
                from: {
                    lineIndex: this.getCaretLineIndex(range.startContainer, range.startOffset),
                    charIndex: this.getCaretCharIndex(range.startContainer, range.startOffset)
                },
                to: {
                    lineIndex: this.getCaretLineIndex(range.endContainer, range.endOffset - 1),
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
           return node.textContent ? node.textContent.length : 0
        }

        // Use a stack to walk through all the child nodes of the `div.ln` node
        // until a node of type TEXT_NODE is found that contains the caret 
        // position. 

        let stack = [...node.childNodes]

        while (stack.length > 0) {
            node = stack.shift()!

            if (!node) {
                continue
            }

            // Use stack.unshift to push all the child nodes to the front of
            // the stack

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


    insertLine(): Caret {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")

        if (caret.to) {
            this.content = deleteRange(this.content, caret.from, caret.to)
        }
        
        this.content = insert(this.content, ["", ""], caret.from)

        this.render()
        
        return {
            from: {
                charIndex: 0,
                lineIndex: caret.from.lineIndex + 1,
            }
        }
    }

    insertText(text: string): Caret {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")
 
        if (caret.to) {
            this.content = deleteRange(this.content, caret.from, caret.to)
        }

        const data = text.split("\n")
        this.content = insert(this.content, data, caret.from)
        this.render()

        return {
            from: {
                charIndex: data.length === 1 ? caret.from.charIndex + data[0].length : data[data.length - 1].length,
                lineIndex: data.length === 1 ? caret.from.lineIndex : caret.from.lineIndex + data.length - 1
            }
        }
    }

    deleteBackward(word?: boolean): Caret {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")

        if (caret.to) {
            this.content = deleteRange(this.content, caret.from, caret.to)

            this.render()

            return { 
                from: caret.from 
            }
        }

        // Check for position at which a backward delete does nothing
        if (caret.from.charIndex === 0 && 
            caret.from.lineIndex === 0) {
            return caret
        }

        // Caret is at start of line so merge with previous line
        if (caret.from.charIndex === 0) {
            const char_index = this.content[caret.from.lineIndex - 1].length

            this.content = mergeLines(this.content, caret.from.lineIndex - 1, caret.from.lineIndex)
            
            this.render()

            return { 
                from: {
                    lineIndex: caret.from.lineIndex - 1,
                    charIndex: char_index
                }
            }
        }

        if (word) {
            const startOfWord = this.startOfWord(caret.from)

            this.content = deleteRange(this.content, startOfWord, caret.from)

            this.render()

            return { 
                from: startOfWord
            }
        }

        // Backward delete a single character
        this.content = removeChars(this.content, caret.from.lineIndex, caret.from.charIndex - 1, caret.from.charIndex)

        this.render()
        
        return {
            from: {
                charIndex: caret.from.charIndex - 1, 
                lineIndex: caret.from.lineIndex,
            }
        }
    }

    deleteForward(): Caret {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")

        if (caret.to) {
            this.content = deleteRange(this.content, caret.from, caret.to)

            this.render()
            
            return { 
                from: caret.from 
            }
        }

        // Check for position at which a forward delete does nothing   
        if (caret.from.charIndex === this.content[caret.from.lineIndex].length && 
            caret.from.lineIndex === this.content.length - 1) {
            return caret
        }

        // Caret it at end of line so merge with following line
        if (caret.from.charIndex === this.content[caret.from.lineIndex].length) {
            this.content = mergeLines(this.content, caret.from.lineIndex, caret.from.lineIndex + 1)
            
            this.render()

            return { 
                from: caret.from
            }
        }

        // Forward delete a single char
        this.content = removeChars(this.content, caret.from.lineIndex, caret.from.charIndex, caret.from.charIndex + 1)

        this.render()

        return {
            from: caret.from
        }
    }

}