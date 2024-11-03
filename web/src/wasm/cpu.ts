import { Canvas } from './canvas'
import { WasmModule, write_bytes_to_module } from './wasm'

const encoder = new TextEncoder()

export class Cpu {
    ptr: number
    module: WasmModule

    constructor(module: WasmModule, input: string) {
        const utf8 = encoder.encode(input)
        const ptr_to_utf8 = write_bytes_to_module(module, utf8)

        this.ptr = module.exports.create_cpu(ptr_to_utf8, utf8.length) as number
        this.module = module
    }

    run(canvas: Canvas, n: number) {
        if (this.ptr < 0) return
        this.module.exports.cpu_run(this.ptr, canvas.ptr, n)
    }

    destroy() {
        if (this.ptr < 0) return
        this.module.exports.destroy_cpu(this.ptr);
        this.ptr = -1;
    }

    is_halted(): boolean {
        if (this.ptr < 0) return true
        return this.module.exports.cpu_is_halted(this.ptr) > 0
    }
}

