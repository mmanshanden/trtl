import { WasmModule, readReturnBytesFromModule } from "./wasm"


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
        const pixels = readReturnBytesFromModule(this.module, ptr)

        return pixels
    }

    clear() {
        this.module.exports.canvas_clear(this.ptr)
    }

    destroy() {
        if (this.ptr < 0) return null

        this.module.exports.destroy_canvas(this.ptr)
        this.ptr = -1
    }
}
