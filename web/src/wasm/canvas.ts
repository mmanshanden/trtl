import { Module, read_return_bytes_from_module } from "./wasm"


export class Canvas {
    ptr: number
    module: Module

    constructor(module: Module, width: number, height: number) {
        this.ptr = module.exports.create_canvas(width, height) as number
        this.module = module
    }

    get_bytes(): Uint8ClampedArray | null {
        if (this.ptr < 0) return null

        const ptr = this.module.exports.canvas_pixels(this.ptr) as number
        const bytes = read_return_bytes_from_module(this.module, ptr)

        return new Uint8ClampedArray(bytes)
    }

    destroy() {
        if (this.ptr < -1) return null

        this.module.exports.destroy_canvas(this.ptr)
        this.ptr = -1
    }
}
