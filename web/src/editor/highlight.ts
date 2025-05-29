import { decode } from "../wasm/benocde"
import { WasmModule, readReturnBytesFromModule, writeBytesToModule } from "../wasm/wasm"

type Style = 'keyword' | 'flow' |  'identifier' | 'number' | 'constant' | 'func' | 'move' | 'comment' | 'error' | 'call'
type Error = 'insert' | 'remove' | 'lint'

interface Fragment {
    value: string
    kind: Style
    error: Error
    hint?: string
}

interface Line {
    fragments: Fragment[]
}

            // Marker::LineBreak => {
            //     lines.push(line);
            //     line = Vec::new();
            // }
            // Marker::Number(str) => {
            //     line.push(Fragment {
            //         value: str.to_string(),
            //         kind: 1,
            //         hint: 0,
            //     });
            // }
            // Marker::Identifier(str) => {
            //     line.push(Fragment {
            //         value: str.to_string(),
            //         kind: 2,
            //         hint: 0,
            //     });
            // }
            // Marker::Keyword(token) => {
            //     line.push(Fragment {
            //         value: token_to_string(token),
            //         kind: 3,
            //         hint: 0,
            //     });
            // }
            // Marker::Control(token) => {
            //     line.push(Fragment { 
            //         value: token_to_string(token), 
            //         kind: 4, 
            //         hint: 0 
            //     });
            // }
            // Marker::Move(token) => {
            //     line.push(Fragment {
            //         value: token_to_string(token),
            //         kind: 5,
            //         hint: 0,
            //     });
            // }
            // Marker::Function(str) => {
            //     line.push(Fragment {
            //         value: str.to_string(),
            //         kind: 10,
            //         hint: 0,
            //     });
            // }
            // Marker::Call(str) => {
            //     line.push(Fragment {
            //         value: str.to_string(),
            //         kind: 11,
            //         hint: 0,
            //     });
            // }
            // Marker::Plain(token) => {
            //     line.push(Fragment {
            //         value: token_to_string(token),
            //         kind: 0,
            //         hint: 0,
            //     });
            // }
            // Marker::Comment(str) => {
            //     line.push(Fragment {
            //         value: str.to_string(),
            //         kind: 20,
            //         hint: 0,
            //     });
            // }
            // Marker::Whitespace(str) => {
            //     line.push(Fragment {
            //         value: str.to_string(),
            //         kind: 0,
            //         hint: 0,
            //     });
            // }
            // Marker::UnexpectedToken { expected: _, actual } => for &token in actual {
            //     line.push(Fragment {
            //         value: token_to_string(token),
            //         kind: 40,
            //         hint: 0
            //     });
            // }

const translate_kind = (kind: number): Style | undefined => {
    if (kind === 1) {
        return 'number'
    } else if (kind === 2) {
        return 'identifier'
    } else if (kind === 3) {
        return 'keyword'
    } else if (kind === 4) {
        return 'flow'  
    } else if (kind === 5) {
        return 'move'  
    } else if (kind === 10) {
        return 'func'
    } else if (kind === 11) {
        return 'call'
    } else if (kind === 20) {
        return 'comment'
    } else if (kind === 40) {
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
