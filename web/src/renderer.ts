import { Canvas } from "./wasm/canvas"
import { Cpu } from "./wasm/cpu"
import { loadModule } from "./wasm/wasm"

const module = await loadModule('../wasm/wasm32-unknown-unknown/release/trtl.wasm')

let cpu: Cpu | null = null
let canvas: Canvas | null = null
let sprite: ImageData | null = null

let renderRequestId: number = 0;

export interface RenderCall {
    input: string,
    width: number,
    height: number
}

const render = () => {
    if (cpu == null || canvas == null || sprite == null) {
        return
    }

    if (cpu.is_halted()) {
        return
    }

    cpu.run(canvas, 4_000)
    const bytes = canvas.get_bytes()

    if (bytes) {
        sprite.data.set(bytes)
        postMessage(sprite)
    }

    renderRequestId = requestAnimationFrame(render)
}

onmessage = async (e) => {
    if (renderRequestId != 0) {
        cancelAnimationFrame(renderRequestId)
    }

    const { input, width, height } = e.data as RenderCall

    cpu?.destroy()
    cpu = new Cpu(module, input)

    if (canvas == null || canvas.width != width && canvas.height != height) {
        canvas?.destroy()
        canvas = new Canvas(module, width, height)
        sprite = new ImageData(width, height)
    } else {
        canvas.clear()
    }

    render()
}

postMessage(1)
