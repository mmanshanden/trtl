import { Module } from '../wasm/wasm'
import { getCaret, selectCurrentLine, setCaret } from './caret'
import { highlight } from './highlight'
import { KeyMap, pressKey, releaseKey } from './keymap'

export interface Editor {
    element: HTMLDivElement
    keyMap: KeyMap
    module: Module
}

const ignoredKeys = ["ArrowUp", "ArrowLeft", "ArrowRight", "ArrowDown", "Control", "Shift", "Alt", "CapsLock"]

export const setContent = ({ module, element }: Editor, ...content: string[]) => {
    const input = content.flatMap(line => line.split('\n'))
    const lines = highlight(module, input)

    let html = ""

    lines.forEach(({ fragments }, lineIndex) => {
        html += `<div class="ln">`

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

export const editor = (element: HTMLDivElement, module: Module): Editor => {
    element.innerHTML = `<div class="code-editor" contenteditable="true" spellcheck="false" />`
    const codeEditor = element.querySelector<HTMLDivElement>('div.code-editor')!

    const state: Editor = {
        element: codeEditor,
        keyMap: { state: new Map() },
        module: module
    }

    codeEditor.focus()

    element.addEventListener("keyup", (e) => {
        const { key, control } = releaseKey(state.keyMap, e)

        if (ignoredKeys.includes(key) || control) {
            return
        }

        let caret = getCaret(state)
        let content = getContent(state)
        setContent(state, ...content)
        if (caret) { 
            setCaret(state, caret.from)
        }
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

    const printDebug = () => {
        const caret = getCaret(state)
        console.log(caret)

        setTimeout(() => {
            printDebug()
        }, 1200);
    }

    //printDebug()

    return state
}
