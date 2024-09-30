import { Editor } from "./editor"


const nodeContentLength = (node: ChildNode): number => {
    return node.textContent?.length ?? 0
}

const positionInNode = (node: ChildNode): number => {
    const selection = document.getSelection()

    if (!selection) {
        return 0
    }

    const range = selection.getRangeAt(0)
    const rel = range.cloneRange()

    rel.selectNodeContents(node)
    rel.setEnd(range.endContainer, range.endOffset)

    return rel.toString().length
}

export interface Caret {
    absoluteIndex: number,
    lineIndex: number,
    charIndex: number
}

export const selectCurrentLine = ({ element, ...editor }: Editor) => {
    const selection = document.getSelection()

    if (!selection) {
        return
    }

    const range = document.createRange()
    const { lineIndex } = getCaret({ element: element, ...editor })

    const line = element.childNodes[lineIndex]

    range.setStart(line, 0)
    range.setEnd(line, 1)

    selection.removeAllRanges()
    selection.addRange(range)
}

export const getCaret = ({ element }: Editor): Caret => {
    const selection = document.getSelection()

    if (!selection) {
        return {
            absoluteIndex: 0,
            lineIndex: 0,
            charIndex: 0
        }
    }

    const range = selection.getRangeAt(0)
    const node = range.endContainer

    let absoluteIndex = 0
    let lineIndex = 0

    for (const line of element.childNodes) {
        if (line.contains(node) || line === node) {
            const length = positionInNode(line)

            return {
                absoluteIndex: absoluteIndex + length,
                charIndex: length,
                lineIndex: lineIndex
            }
        }

        absoluteIndex += nodeContentLength(line)
        lineIndex += 1
    }

    return {
        absoluteIndex: 0,
        lineIndex: 0,
        charIndex: 0
    }
}

export const setCaret = ({ element }: Editor, { charIndex, lineIndex }: Caret) => {
    const selection = window.getSelection();

    if (!selection) {
        return
    }

    const range = document.createRange();

    let node = element.childNodes[lineIndex]
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

        const length = nodeContentLength(node)

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
