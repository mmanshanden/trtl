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

        let startTime, endTime;
        startTime = performance.now()

        const ptr = this.module.exports.canvas_pixels(this.ptr)
        
        endTime = performance.now()

        console.log(`Call to wasm.get_pixels ${endTime - startTime} milliseconds`)
        
        startTime = performance.now()

        const pixels = read_return_bytes_from_module(this.module, ptr)
        
        endTime = performance.now()

        console.log(`Call to wasm.read ${endTime - startTime} milliseconds`)

        return pixels
    }

    destroy() {
        if (this.ptr < 0) return null

        this.module.exports.destroy_canvas(this.ptr)
        this.ptr = -1
    }
}
