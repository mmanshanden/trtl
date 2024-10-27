import { Canvas } from "./wasm/canvas"
import { Cpu } from "./wasm/cpu"
import { load_module } from "./wasm/wasm"

const module = await load_module('../wasm/wasm32-unknown-unknown/debug/trtl.wasm')

let cpu: Cpu | null = null
let canvas: Canvas | null = null

export interface RenderCall {
    input: string,
    width: number,
    height: number
}

onmessage = async (e) => {
    if (cpu != null) {
        cpu.destroy()
    }

    if (canvas != null) {
        canvas.destroy()
    }

    const { input, width, height } = e.data as RenderCall

    cpu = new Cpu(module, input)
    canvas = new Canvas(module, width, height)
    
    cpu.run(canvas, 100000);

    const bytes = canvas.get_bytes()

    if (bytes) {
        const image = new ImageData(bytes, width, height)
        postMessage(image)
    }
}
