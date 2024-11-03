import { WasmModule, read_return_bytes_from_module } from "./wasm"


export class Canvas {
    ptr: number
    width: number
    height: number
    module: WasmModule

    constructor(module: WasmModule, width: number, height: number) {
        this.ptr = module.exports.create_canvas(width, height) as number
        this.width = width
        this.height = height
        this.module = module
    }

    get_bytes(): Uint8Array | null {
        if (this.ptr < 0) return null
        const ptr = this.module.exports.canvas_pixels(this.ptr)
        const pixels = read_return_bytes_from_module(this.module, ptr, "pixels")

        return pixels
    }

    clear() {
        this.module.exports.canvas_clear(this.ptr)
    }

    destroy() {
        if (this.ptr < 0) return null

        console.log("destroying canvas")
        this.module.exports.destroy_canvas(this.ptr)
        this.ptr = -1
    }
}
