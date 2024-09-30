import { getCaret, selectCurrentLine, setCaret } from './caret'
import { highlight } from './highlight'
import { KeyMap, pressKey, releaseKey } from './keymap'

export interface Editor {
    element: HTMLDivElement
    keyMap: KeyMap
}

const ignoredKeys = ["ArrowUp", "ArrowLeft", "ArrowRight", "ArrowDown", "Control", "Shift", "Alt", "CapsLock"]

const setContent = ({ element }: Editor, ...content: string[]) => {
    const input = content.flatMap(line => line.split('\n'))
    const lines = highlight(input)

    let html = ""

    lines.forEach(({ fragments }, lineIndex) => {
        html += `<div class="editor-line">`

        fragments.forEach(({ value, style }) => {
            if (style) {
                html += `<span class="${style}">${value}</span>`
            } else if (value === " ") {
                // Simply inserting a ` ` token has proven to be buggy, at least in Firefox. 
                // Inserting a `&nbsp;` token instead gives no issues.
                html += '&nbsp;';
            } else {
                html += value
            }
        })

        if (fragments.length == 0 || lineIndex === lines.length - 1) {
            html += '<br>'
        }

        html += `</div>`
    })

    element.innerHTML = html
}

const getContent = ({ element }: Editor) => {
    const lines = Array.from(element.childNodes)
    return lines.map(node => node.textContent ?? "")
}

export const editor = (element: HTMLDivElement): Editor => {
    element.innerHTML = `<div class="code-editor" contenteditable="true" spellcheck="false" />`
    const codeEditor = element.querySelector<HTMLDivElement>('div.code-editor')!

    const state: Editor = {
        element: codeEditor,
        keyMap: { state: new Map() }
    }

    setContent(state, "")

    element.addEventListener("keyup", (e) => {
        const { key, control } = releaseKey(state.keyMap, e)

        if (ignoredKeys.includes(key) || control) {
            return
        }

        let caret = getCaret(state)
        let content = getContent(state)
        setContent(state, ...content)
        setCaret(state, caret)
    })

    element.addEventListener("keydown", (e) => {
        pressKey(state.keyMap, e)
    })

    element.addEventListener("cut", (e) => {
        selectCurrentLine(state)
    })

    element.addEventListener("copy", (e) => {
        selectCurrentLine(state)
    })

    element.addEventListener("paste", (e) => {
        e.preventDefault()
    })

    return state
}
