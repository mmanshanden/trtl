import { decode, DecodeResult } from "../wasm/benocde"
import { Module, read_return_bytes_from_module, write_bytes_to_module } from "../wasm/wasm"

type Style = 'keyword' | 'flow' |  'identifier' | 'number' | 'constant'
type Error = 'insert' | 'remove'

interface Fragment {
    value: string
    kind: Style
    error: Error
    hint?: string
}

interface Line {
    fragments: Array<Fragment>
}

const translate_kind = (kind: number): Style | undefined => {
    if (kind === 1) {
        return 'keyword'
    } else if (kind === 2) {
        return 'flow'
    } else if (kind === 3) {
        return 'constant'
    } else if (kind === 4) {
        return 'number'
    } else if (kind === 5) {
        return 'identifier'
    } else {
        return undefined
    }
}

const translate_hint = (lint: number): Error | undefined => {
    if (lint > 1) {
        return 'insert'
    } else if (lint === 1) {
        return 'remove'
    } else {
        return undefined
    }
}

export const highlight = (module: Module, input: string): Line[] => {
    const decoder = new TextDecoder('utf-8')

    const utf8 = new TextEncoder().encode(input)
    const ptr_to_utf8 = write_bytes_to_module(module, utf8)
    const ptr_to_output = module.exports.high(ptr_to_utf8, utf8.length) as number;
    const output = read_return_bytes_from_module(module, ptr_to_output);

    const [lines, hints] = decode(output).value

    return lines.map((line: Uint8Array[]) => {
        return {
            fragments: line.map((fragment: Uint8Array) => {
                let kind = fragment[0]
                let hint = fragment[1]
                let utf8 = fragment.subarray(2, fragment.length)

                return {
                    value: decoder.decode(utf8),
                    kind: translate_kind(kind),
                    error: translate_hint(hint),
                    hint: hint > 1 ? decoder.decode(hints[hint]) : undefined
                }
            })
        }
    })
}
