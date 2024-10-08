const ZERO = 48
const ONE = 49
const TWO = 50
const THREE = 51
const FOUR = 52
const FIVE = 53
const SIX = 54
const SEVEN = 55
const EIGHT = 56
const NINE = 57
const COLON = 58
const CHAR_E = 101
const CHAR_L = 108

export interface DecodeResult {
    value: any,
    end: number
}

const decode_string = (bytes: Uint8Array, offset: number): DecodeResult => {
    // read integer marking length
    let l = 0
    while (bytes[offset] != COLON && offset < bytes.length) {
        l = (bytes[offset] - ZERO) + l * 10
        offset = offset + 1
    }

    // read colon
    offset = offset + 1

    return {
        value: bytes.subarray(offset, offset + l),
        end: offset + l
    }
}

const decode_list = (bytes: Uint8Array, offset: number): DecodeResult => {
    let pos = offset + 1
    let values = []

    while (bytes[pos] != CHAR_E) {
        const { value, end } = decode(bytes, pos)
        values.push(value)
        pos = end
    }

    return {
        value: values,
        end: pos + 1
    }
}

export const decode = (bytes: Uint8Array, offset = 0): DecodeResult => {
    const ident = bytes[offset]

    switch (ident) {
        case CHAR_L:
            return decode_list(bytes, offset)
        case ZERO:
            return decode_string(bytes, offset)
        case ONE:
            return decode_string(bytes, offset)
        case TWO:
            return decode_string(bytes, offset)
        case THREE:
            return decode_string(bytes, offset)
        case FOUR:
            return decode_string(bytes, offset)
        case FIVE:
            return decode_string(bytes, offset)
        case SIX:
            return decode_string(bytes, offset)
        case SEVEN:
            return decode_string(bytes, offset)
        case EIGHT:
            return decode_string(bytes, offset)
        case NINE:
            return decode_string(bytes, offset)
    }

    throw new Error("Error while decoding. Reading " + ident + " at " + offset)
}
