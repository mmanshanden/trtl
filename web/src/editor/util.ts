import { Position } from "./editor"

export function firstLine(buffer: string[]): string {
    if (buffer.length === 0) {
        return ""
    }

    return buffer[0]
}

export function lastLine(buffer: string[]): string {
    if (buffer.length === 0) {
        return ""
    }

    return buffer[buffer.length - 1]
}

export function append(buffer: string[], append: string[]): string[] {
    if (!append.length) return buffer

    const [first, ...remainder] = append

    return [
        ...buffer.slice(0, -1),
        lastLine(buffer) + first,
        ...remainder
    ]
}

export function split(buffer: string[], position: Position): [string[], string[]] {
    const before = buffer.slice(0, position.lineIndex + 1)
    const after = buffer.slice(position.lineIndex)

    // Remove the characters that come after the give position
    before[before.length - 1] = before[before.length - 1].slice(0, position.charIndex)

    // And also remove the character before the given position
    if (after.length) after[0] = after[0].slice(position.charIndex)

    return [before, after]
}


export function keepRange(buffer: string[], from: Position, to: Position): string[] {
    const [content, _after] = split(buffer, to)
    const [_before, middle] = split(content, from)

    return middle
}

export function deleteRange(buffer: string[], from: Position, to: Position): string[] {
    const [content, after] = split(buffer, to)
    const [before, _middle] = split(content, from)

    return append(before, after)
}

export function insert(buffer: string[], data: string[], position: Position): string[] {
    const [before, after] = split(buffer, position)

    return append(append(before, data), after)
}

export function mergeLines(buffer: string[], index_from: number, index_to: number): string[] {
    const before = buffer.slice(0, index_from)
    const after = buffer.slice(index_to + 1)

    const merged = buffer.slice(index_from, index_to + 1).join("")

    return [
        ...before,
        merged,
        ...after
    ]
}

export function removeChars(buffer: string[], lineIndex: number, from: number, to: number): string[] {
    const before = buffer.slice(0, lineIndex)
    const after = buffer.slice(lineIndex + 1)

    // UTF8 safe method of removing chars
    const line = buffer[lineIndex]
    const sliced = line.slice(0, from) + line.slice(to)

    return [
        ...before,
        sliced,
        ...after
    ]
}
