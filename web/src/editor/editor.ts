import { WasmModule } from "../wasm/wasm"
import { highlight } from "./highlight"


const WORD_BOUNDARY = /[\s-\+\*\/\\\(\)\.\[\]\{\}\;\=\:\<\>\?\%\^\!\@\#\$\&\`\'\"]/

export interface Position {
    lineIndex: number,
    charIndex: number
}

export interface Caret {
    from: Position
    to?: Position
}


class Content {
    private lines: string[]

    constructor(content: string[]) {
        this.lines = content
    }

    lineCount(): number {
        return this.lines.length
    }

    lengthOfLine(line_index: number): number {
        return this.lines[line_index].length
    }

    string(): string {
        return this.lines.join("\n")
    }

    private first(): string {
        if (this.lines.length === 0) {
            return ""
        }

        return this.lines[0]
    }

    private last(): string {
        if (this.lines.length === 0) {
            return ""
        }

        return this.lines[this.lines.length - 1]
    }

    private prepend(content: Content): Content {
        const remainder = content.lines.slice(0, -1)
        const last = content.lines.at(-1) ?? ""

        return new Content([
            ...remainder,
            last + this.lines[0],
            ...this.lines.slice(1)
        ])
    }

    private append(content: Content): Content {
        const [first, ...remainder] = content.lines

        return new Content([
            ...this.lines.slice(0, -1),
            this.last() + first,
            ...remainder
        ])
    }
    
    private split(position: Position): [Content, Content] {
        const before = this.lines.slice(0, position.lineIndex + 1)
        const after = this.lines.slice(position.lineIndex)

        // Remove the characters that come after the give position
        before[before.length - 1] = before[before.length - 1].slice(0, position.charIndex)

        // And also remove the character before the given position
        after[0] = after[0].slice(position.charIndex)

        return [new Content(before), new Content(after)]
    }

    startOfWord(position: Position): Position {
        const line = this.lines[position.lineIndex]
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


    keepRange(from: Position, to: Position): Content {
        const [content, _after] = this.split(to)
        const [_before, middle] = content.split(from)

        return middle
    }

    deleteRange(from: Position, to: Position): Content {
        const [content, after] = this.split(to)
        const [before, _middle] = content.split(from)

        return before.append(after)
    }

    insert(data: string[], position: Position): Content { 
        const [before, after] = this.split(position)
        const insert = new Content(data)

        return before.append(insert).append(after)
    }

    mergeLines(index_from: number, index_to: number): Content {
        const lines_before = this.lines.slice(0, index_from)
        const lines_after = this.lines.slice(index_to + 1)

        const middle = this.lines.slice(index_from, index_to + 1).join("")

        return new Content([
            ...lines_before,
            middle,
            ...lines_after
        ])
    }

    removeChars(line: number, from: number, to: number): Content {
        return new Content([
            ...this.lines.slice(0, line),
            this.lines[line].substring(0, from) + this.lines[line].substring(to),
            ...this.lines.slice(line + 1)
        ])
    }
}


export class Editor extends EventTarget {
    gutter: HTMLDivElement
    input: HTMLDivElement
    module: WasmModule
    private content: Content

    constructor(parent: HTMLDivElement, module: WasmModule) {
        super()

        const [gutter, input] = this.createElement(parent) 
        this.gutter = gutter
        this.input = input
        this.module = module
        this.content = new Content([""])

        this.input.addEventListener('beforeinput', (e) => {
            this.handleInputEvent(e as InputEvent)
            e.preventDefault()
        })

        this.input.addEventListener('input', (e) => {
            const event = e as InputEvent

            if (event.isComposing) return
            if (event.inputType != 'insertCompositionText') return

            this.insertText(event.data!)
            e.preventDefault()
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
                <div class="input" contenteditable="true" spellcheck="false"></div>
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
        const input = this.content.string()
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

    private handleInputEvent(e: InputEvent) {
        const caret = this.getCaret()

        if (caret == null) {
            throw new Error("INPUT_WITHOUT_CARET")
        }

        switch (e.inputType) {
            case "deleteContentForward":
                this.deleteForward()
                break
            case "deleteContentBackward":
                this.deleteBackward()
                break
            case "deleteWordBackward":
                this.deleteBackward(true)
                break
            case "insertParagraph":
            case "insertLineBreak":
                this.insertLine()
                break
            case "insertText":
                this.insertText(e.data!)
                break

        }
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

    focus() {
        this.input.focus()
    }

    setContent(content: string) {
        this.content = new Content(content.split("\n"))

        this.render()
        this.setCaret({
            from: {
                charIndex: 0,
                lineIndex: 0
            }
        })
    }

    getContent(): string {
        return this.content.string()
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
                        lineIndex: this.content.lineCount() - 1,
                        charIndex: this.content.lengthOfLine(this.content.lineCount() - 1)
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
           return node.textContent?.length ?? 0
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


    insertLine() {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")

        if (caret.to) {
            this.content = this.content.deleteRange(caret.from, caret.to)
        }
        
        this.content = this.content.insert(["", ""], caret.from)

        this.render()
        this.setCaret({
            from: {
                charIndex: 0,
                lineIndex: caret.from.lineIndex + 1,
            }
        })
    }

    insertText(text: string) {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")
 
        if (caret.to) {
            this.content = this.content.deleteRange(caret.from, caret.to)
        }

        this.content = this.content.insert([text], caret.from)

        this.render()
        this.setCaret({
            from: {
                charIndex: caret.from.charIndex + text.length,
                lineIndex: caret.from.lineIndex
            }
        })
    }

    deleteBackward(word?: boolean) {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")

        if (caret.to) {
            this.content = this.content.deleteRange(caret.from, caret.to)

            this.render()
            this.setCaret({ 
                from: caret.from 
            })

            return
        }

        // Check for position at which a backward delete does nothing
        if (caret.from.charIndex === 0 && 
            caret.from.lineIndex === 0) {
            return
        }

        // Caret is at start of line so merge with previous line
        if (caret.from.charIndex === 0) {
            const char_index = this.content.lengthOfLine(caret.from.lineIndex - 1)

            this.content = this.content.mergeLines(caret.from.lineIndex - 1, caret.from.lineIndex)
            
            this.render()
            this.setCaret({ 
                from: {
                    lineIndex: caret.from.lineIndex - 1,
                    charIndex: char_index
                }
            })

            return
        }

        if (word) {
            const startOfWord = this.content.startOfWord(caret.from)

            this.content = this.content.deleteRange(startOfWord, caret.from)

            this.render()
            this.setCaret({ 
                from: startOfWord
            })

            return
        }

        // Backward delete a single character
        this.content = this.content.removeChars(caret.from.lineIndex, caret.from.charIndex - 1, caret.from.charIndex)

        this.render()
        this.setCaret({
            from: {
                charIndex: caret.from.charIndex - 1, 
                lineIndex: caret.from.lineIndex,
            }
        })
    }

    deleteForward() {
        const caret = this.getCaret()
        if (!caret) throw new Error("NO_CARET")

        if (caret.to) {
            this.content = this.content.deleteRange(caret.from, caret.to)

            this.render()
            this.setCaret({ 
                from: caret.from 
            })

            return
        }

        // Check for position at which a forward delete does nothing   
        if (caret.from.charIndex === this.content.lengthOfLine(caret.from.lineIndex) && 
            caret.from.lineIndex === this.content.lineCount() - 1) {
            return
        }

        // Caret it at end of line so merge with following line
        if (caret.from.charIndex === this.content.lengthOfLine(caret.from.lineIndex)) {
            this.content = this.content.mergeLines(caret.from.lineIndex, caret.from.lineIndex + 1)
            
            this.render()
            this.setCaret({ 
                from: caret.from
            })

            return
        }

        // Forward delete a single char
        this.content = this.content.removeChars(caret.from.lineIndex, caret.from.charIndex, caret.from.charIndex + 1)

        this.render()
        this.setCaret({
            from: caret.from
        })
    }

}