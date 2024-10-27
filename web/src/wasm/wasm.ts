
type WasmPtr = number

export interface WasmExports {
    malloc: (len: number) => WasmPtr,
    mfree: (ptr: WasmPtr, len: number) => void,
    
    syntax_fragments: (ptr: WasmPtr, len: number) => WasmPtr,
    
    create_cpu: (ptr: WasmPtr, len: number) => WasmPtr,
    create_canvas: (width: number, height: number) => WasmPtr,

    destroy_cpu: (ptr: WasmPtr) => void,
    destroy_canvas: (ptr: WasmPtr) => void,

    cpu_run: (ptr_cpu: WasmPtr, ptr_canvas: WasmPtr, n: number) => void,
    
    canvas_pixels: (ptr_cpu: WasmPtr) => WasmPtr
}

export interface WasmModule {
    exports: WasmExports
    memory: WebAssembly.Memory
}

const read_bytes_from_module = (module: WasmModule, ptr: number, len: number, cap: number): Uint8Array => {
    const mem = new Uint8Array(module.memory.buffer, ptr, len);
    const bytes = new Uint8Array(mem.length)

    bytes.set(mem)
    module.exports.mfree(ptr, cap)

    return bytes
}

export const read_return_bytes_from_module = (module: WasmModule, ptr: number): Uint8Array => {
    const mem = new Uint32Array(module.memory.buffer, ptr, 3)
    const buffer_ptr = mem[0]
    const buffer_len = mem[1]
    const buffer_cap = mem[2]
    module.exports.mfree(ptr, 12)

    return read_bytes_from_module(module, buffer_ptr, buffer_len, buffer_cap)
}

export const write_bytes_to_module = (module: WasmModule, bytes: Uint8Array): number => {
    const ptr = module.exports.malloc(bytes.length) as number
    new Uint8Array(module.memory.buffer).set(bytes, ptr)
    return ptr
}

export const load_module = async (path: string): Promise<WasmModule> => {
    let module: WasmModule | null = null

    const env = {
        alert: (ptr: WasmPtr, len: number, cap: number) => {
            if (!module) return
            const bytes = read_bytes_from_module(module, ptr, len, cap)
            const decoded = new TextDecoder('utf-8').decode(bytes)
            console.error(decoded)
        },
        print: (ptr: WasmPtr, len: number, cap: number) => {
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
        exports: (wasm.instance.exports as unknown) as WasmExports,
        memory: wasm.instance.exports.memory as WebAssembly.Memory
    }

    return module
}




