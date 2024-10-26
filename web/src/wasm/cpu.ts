import { Module, read_return_bytes_from_module, write_bytes_to_module } from './wasm'


export interface Cpu {
    ptr: number
    module: Module
}

export const createCpu = (module: Module, input: string): Cpu | null => {
    const utf8 = new TextEncoder().encode(input)
    const ptr_to_utf8 = write_bytes_to_module(module, utf8)

    const ptr = module.exports.create_cpu(ptr_to_utf8, utf8.length);

    if (!ptr) {
        return null
    }

    return {
        ptr,
        module
    }
}

export const destroyCpu = ({ptr, module}: Cpu) => {
    module.exports.destroy_cpu(ptr)
} 

export const cpuExec = ({ptr, module}: Cpu, n: number): Uint8Array => {
    const canvas_bytes_ptr = module.exports.cpu_exec(ptr, 512, 512, n) as number
    return read_return_bytes_from_module(module, canvas_bytes_ptr);
}
