import { Editor } from "./editor"


const elementLength = (node: ChildNode): number => {
    return node.textContent?.length ?? 0
}

const positionInElement = (range: Range, node: ChildNode, side: 'start' | 'end'): number => {
    const rel = range.cloneRange()

    rel.selectNodeContents(node)

    if (side === 'start') {
        rel.setEnd(range.startContainer, range.startOffset) 
    } else {
        rel.setEnd(range.endContainer, range.endOffset)
    }

    return rel.toString().length
}

const positionInDocument = (selection: Selection, document: HTMLDivElement, side: 'start' | 'end'): Position => {
    const range = selection.getRangeAt(0)
    const node = side === 'start' ? range.startContainer : range.endContainer

    let absoluteIndex = 0
    let lineIndex = 0

    for (const line of document.childNodes) {
        if (line.contains(node) || line === node) {
            const length = positionInElement(range, line, side)

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

export interface Position {
    absoluteIndex: number,
    lineIndex: number,
    charIndex: number
}

export interface Caret {
    from: Position
    to?: Position
}

export const selectCurrentLine = ({ element, ...editor }: Editor) => {
    const selection = document.getSelection()

    if (!selection) {
        return
    }

    const range = document.createRange()
    const { lineIndex } = getCaret({ element: element, ...editor })!.from

    const line = element.childNodes[lineIndex]

    range.setStart(line, 0)
    range.setEnd(line, 1)

    selection.removeAllRanges()
    selection.addRange(range)
}

export const getCaret = ({ element }: Editor): Caret | null => {
    const selection = document.getSelection()

    if (!selection) {
        return null
    }

    let range = selection.getRangeAt(0)

    if (range.startOffset === range.endOffset && range.startContainer === range.endContainer) {
        return {
            from: positionInDocument(selection, element, 'start')
        }
    }

    return {
        from: positionInDocument(selection, element, 'start'),
        to: positionInDocument(selection, element, 'end')
    }
}

export const setCaret = ({ element }: Editor, { charIndex, lineIndex }: Position) => {
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
