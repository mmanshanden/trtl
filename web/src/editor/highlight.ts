import { decode, DecodeResult } from "../wasm/benocde"
import { WasmModule, readReturnBytesFromModule, writeBytesToModule } from "../wasm/wasm"

type Style = 'keyword' | 'flow' |  'identifier' | 'number' | 'constant' | 'func' | 'instruction' | 'comment' | 'error'
type Error = 'insert' | 'remove' | 'lint'

interface Fragment {
    value: string
    kind: Style
    error: Error
    hint?: string
}

interface Line {
    fragments: Array<Fragment>
}

// Tag::Whitespace(str) => (vec![str.to_string()], 0),
// Tag::Plain(token) => (vec![token_to_string(token)], 0),
// Tag::Comment(str) => (vec![str.to_string()], 1),
// Tag::Call(name) => (vec![name.to_string()], 2),
// Tag::Move(token) => (vec![token_to_string(token)], 3),
// Tag::Identifier(name) => (vec![name.to_string()], 4),
// Tag::Number(num) => (vec![num.to_string()], 5),
// Tag::Keyword(token) => (vec![token_to_string(token)], 6),

// Tag::UnexpectedToken { expected: _, actual } => {
//     (actual.iter().map(|&token| token_to_string(token)).collect(), 20)
// }

const translate_kind = (kind: number): Style | undefined => {
    if (kind === 1) {
        return 'comment'
    } else if (kind === 2) {
        return 'func'
    } else if (kind === 3) {
        return 'instruction'
    } else if (kind === 4) {
        return 'identifier'  
    } else if (kind === 5) {
        return 'number'
    } else if (kind === 6) {
        return 'keyword'
    } else if (kind === 20) {
        return 'error'
    } else {
        return undefined
    }
}

const translate_hint = (lint: number): Error | undefined => {
    if (lint > 1) {
        return 'lint'
    } else if (lint === 1) {
        return 'lint'
    } else {
        return undefined
    }
}

export const highlight = (module: WasmModule, input: string): Line[] => {
    const decoder = new TextDecoder('utf-8')

    const utf8 = new TextEncoder().encode(input)
    const ptr_to_utf8 = writeBytesToModule(module, utf8)
    const ptr_to_output = module.exports.syntax_fragments(ptr_to_utf8, utf8.length);
    const output = readReturnBytesFromModule(module, ptr_to_output);

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
