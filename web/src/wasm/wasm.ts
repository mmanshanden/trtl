
type Export = (...args: number[]) => (void | number)

interface Module {
    exports: Record<string, Export>
    memory: WebAssembly.Memory
}


const read_string_from_module = (module: Module, ptr: number, len: number) => {
    const mem = new Uint8Array(module.memory.buffer);
    const buffer = mem.subarray(ptr, ptr + len);
    
    return new TextDecoder('utf-8').decode(buffer);
}

export const read_return_string_from_module = (module: Module, ptr: number) => {
    const mem = new Uint8Array(module.memory.buffer);
    const ret_ptr = mem[ptr + 0] | mem[ptr + 1] << 8 | mem[ptr + 2] << 16 | mem[ptr + 3] << 24;
    const ret_len = mem[ptr + 4] | mem[ptr + 5] << 8 | mem[ptr + 6] << 16 | mem[ptr + 7] << 24;

    module.exports.mfree(ptr, 8)

    const value = read_string_from_module(module, ret_ptr, ret_len)

    module.exports.mfree(ret_ptr, ret_len)

    return value
}

export const loadModule = async (path: string) => {
    let module: Module | null = null

    const env = {
        alert: (ptr: number, len: number) => {
            if (!module) return
            const str = read_string_from_module(module, ptr, len)
            console.error(str)
        },
        print: (ptr: number, len: number) => {
            if (!module) return
            const str = read_string_from_module(module, ptr, len)
            console.info(str)
        }
    }

    const data = fetch(path);
    const wasm = await WebAssembly.instantiateStreaming(data, {
        env: env
    })

    module = {
        exports: wasm.instance.exports as Record<string, Export>,
        memory: wasm.instance.exports.memory as WebAssembly.Memory
    }

    return module
}




