import { Module, read_return_bytes_from_module, write_bytes_to_module } from "../wasm/wasm"

type Style = 'keyword' | 'identifier' | 'number'

interface Fragment {
    value: string
    style?: Style
}

interface Line {
    fragments: Array<Fragment>
}

const translate_kind_to_style = (kind: number): Style | undefined => {
    switch (kind) {
        case 1: return 'keyword'
        case 4: return 'identifier'
        default: return undefined
    }
}



export const highlight = (module: Module, input: string[]): Line[] => {
    let lines: Line[] = input.map(line => {
        const utf8 = new TextEncoder().encode(line)
        const ptr_to_utf8 = write_bytes_to_module(module, utf8)
        const ptr_to_frag_bytes = module.exports.high(ptr_to_utf8, utf8.length) as number;
        const frag_bytes = read_return_bytes_from_module(module, ptr_to_frag_bytes);
        const frags = new Uint32Array(frag_bytes.buffer)

        let fragments: Array<Fragment> = []
        let decoder = new TextDecoder('utf-8')

        for (let i = 0; i < frags.length; i += 3) {
            const from = frags[i]
            const to = frags[i + 1]
            const kind = frags[i + 2]

            fragments.push({
                value: decoder.decode(utf8.subarray(from, to)),
                style: translate_kind_to_style(kind)
            })
        }

        return {
            fragments: fragments
        }
    })

    return lines
}