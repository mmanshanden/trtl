import { WasmModule } from '../wasm/wasm'
import { EventBus } from './bus'
import { highlight } from './highlight'

export interface Position {
    lineIndex: number,
    charIndex: number,
    globalIndex: number
}

export interface Caret {
    from: Position
    to: Position | null
}

type Content = string[]

export class Editor {
    mirror: HTMLDivElement
    input: HTMLTextAreaElement
    gutter: HTMLDivElement
    module: WasmModule
    bus: EventBus

    constructor(parent: HTMLDivElement, module: WasmModule, bus: EventBus) {
        const [mirror, input, gutter] = this.createElement(parent) 
        this.mirror = mirror
        this.input = input
        this.gutter = gutter
        this.module = module
        this.bus = bus

        this.input.addEventListener('input', (e) => {
            this.highlight()

            this.bus.publish('contentChanged', { editor: this, content: this.getContent() })
        })

        this.input.addEventListener('scroll', (e) => {
            this.mirror.scrollTop = this.input.scrollTop
            this.gutter.scrollTop = this.input.scrollTop
        })

        this.input.addEventListener('copy', (e) => {
            const caret = this.getCaret()
            if (!caret) return
            if (caret.to) return

            this.writeCurrentLineToClipboard()
        })

        this.input.addEventListener('cut', (e) => {
            const caret = this.getCaret()
            if (!caret) return
            if (caret.to) return
            
            this.writeCurrentLineToClipboard()
            this.removeLine(caret.from.lineIndex)
            this.setCaretPosition(caret.from.lineIndex, 0)
        })
    }

    private createElement(parent: HTMLDivElement): [HTMLDivElement, HTMLTextAreaElement, HTMLDivElement] {
        parent.innerHTML = `
            <div class="code-editor h-stack">
                <div class="gutter"></div>
                <div class="content flex-1">
                    <textarea class="input" spellcheck="false"></textarea>
                    <div class="mirror"><div class="ln"></div></div>
                </div>
            </div>`

        const mirror = parent.querySelector<HTMLDivElement>('div.mirror')!
        const input = parent.querySelector<HTMLTextAreaElement>('textarea.input')!
        const gutter = parent.querySelector<HTMLDivElement>('div.gutter')!

        return [mirror, input, gutter]
    }

    private highlight() {
        const input = this.getContent().join('\n')
        const lines = highlight(this.module, input)
        let mirror = ""
        let gutter = ""

        lines.forEach(({ fragments }, lineIndex) => {
            mirror += `<div class="ln">`
            gutter += `<div class="ln">${lineIndex + 1}</div>`
    
            fragments.forEach(({ value, kind, error, hint }) => {
                if (value == "") {
                    mirror += '<br>'
                } else if (kind || error || hint) {
                    let classes = [kind, error].filter(Boolean).join(" ")
                    let title = Boolean(hint) ? hint : ""
                    mirror += `<span class="${classes}" title="${title}">${value}</span>`
                } else {
                    mirror += value
                }
            })

            mirror += `</div>`
        })
    
        this.mirror.innerHTML = mirror
        this.gutter.innerHTML = gutter
    }

    focus() {
        this.input.focus()
    }

    private getIndexFromPosition(lineIndex: number, charIndex: number): number {
        const content = this.getContent()
        let globalIndex = 0

        for (let i = 0; i < lineIndex; i++) {
            globalIndex += content[i].length + 1
        }

        return globalIndex + charIndex
    }

    private getPositionFromIndex(index: number): Position {
        const content = this.getContent()
        let globalIndex = 0
        let lineIndex = 0
        let charIndex = 0

        while (globalIndex < index) {
            const line = content[lineIndex]
            const length = line.length + 1

            if (globalIndex + length > index) {
                charIndex = index - globalIndex
                break
            }

            globalIndex += length
            lineIndex += 1
        }

        return { lineIndex, charIndex, globalIndex }
    }

    getCaret(): Caret | null {
        const start = this.input.selectionStart;

        if (!start) {
            return null
        }

        const end = this.input.selectionEnd;

        return {
            from: this.getPositionFromIndex(start),
            to: start != end ? this.getPositionFromIndex(end) : null
        }
    }

    setCaretPosition(lineIndex: number, charIndex: number) {
        const globalIndex = this.getIndexFromPosition(lineIndex, charIndex)
        this.input.selectionStart = globalIndex
        this.input.selectionEnd = globalIndex
    }

    setCaret(caret: Caret) {
        const { from, to } = caret
        this.input.selectionStart = from.globalIndex
        this.input.selectionEnd = to ? to.globalIndex : from.globalIndex
    }

    selectCurrentLine() {
        const caret = this.getCaret()
        if (!caret) return

        const content = this.getContent()
        const line = content[caret.from.lineIndex]
        const start = content.slice(0, caret.from.lineIndex).join('\n').length
        const end = start + line.length + 1

        this.input.selectionStart = start
        this.input.selectionEnd = end
    }

    writeCurrentLineToClipboard() {
        const caret = this.getCaret()
        if (!caret) return

        const content = this.getContent()
        const line = content[caret.from.lineIndex]

        navigator.clipboard.writeText(line)
    }

    private removeLine(lineIndex: number) {
        const newContent =  this.getContent().filter((_, i) => i != lineIndex)
        this.setContent(...newContent)
    }

    setContent(...content: Content) {
        this.input.value = content.join('\n')
        this.highlight()

        this.bus.publish('contentChanged', { editor: this, content })
    }

    getContent(): Content {
        const content = this.input.value ?? ""
        const lines = content.split('\n')
        return lines
    }
}

