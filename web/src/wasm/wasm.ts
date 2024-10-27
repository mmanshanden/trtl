
type Export = (...args: number[]) => (void | number)

export interface Module {
    exports: Record<string, Export>
    memory: WebAssembly.Memory
}

const read_bytes_from_module = (module: Module, ptr: number, len: number, cap: number): Uint8Array => {
    const mem = new Uint8Array(module.memory.buffer, ptr, len);
    const bytes = new Uint8Array(mem.length)

    bytes.set(mem)
    module.exports.mfree(ptr, cap)

    return bytes
}

export const read_return_bytes_from_module = (module: Module, ptr: number): Uint8Array => {
    const mem = new Uint8Array(module.memory.buffer);
    const ret_ptr = mem[ptr + 0] | mem[ptr + 1] << 8 | mem[ptr + 2] << 16 | mem[ptr + 3] << 24;
    const ret_len = mem[ptr + 4] | mem[ptr + 5] << 8 | mem[ptr + 6] << 16 | mem[ptr + 7] << 24;
    const ret_cap = mem[ptr + 8] | mem[ptr + 9] << 8 | mem[ptr + 10] << 16 | mem[ptr + 11] << 24;
    module.exports.mfree(ptr, 12)

    console.log("reading", ret_ptr, ret_len)

    return read_bytes_from_module(module, ret_ptr, ret_len, ret_cap)
}

export const write_bytes_to_module = (module: Module, bytes: Uint8Array): number => {
    const ptr = module.exports.malloc(bytes.length) as number
    new Uint8Array(module.memory.buffer).set(bytes, ptr)
    return ptr
}

export const load_module = async (path: string): Promise<Module> => {
    let module: Module | null = null

    const env = {
        alert: (ptr: number, len: number, cap: number) => {
            if (!module) return
            const bytes = read_bytes_from_module(module, ptr, len, cap)
            const decoded = new TextDecoder('utf-8').decode(bytes)
            console.error(decoded)
        },
        print: (ptr: number, len: number, cap: number) => {
            if (!module) return
            const bytes = read_bytes_from_module(module, ptr, len, cap)
            const decoded = new TextDecoder('utf-8').decode(bytes)
            console.log(decoded)
        }
    }

    const response = await fetch(path);
    const data = await response.arrayBuffer()
    const wasm = await WebAssembly.instantiate(data, {
        env: env
    })

    module = {
        exports: wasm.instance.exports as Record<string, Export>,
        memory: wasm.instance.exports.memory as WebAssembly.Memory
    }

    return module
}




