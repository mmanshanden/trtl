import { Canvas } from "./wasm/canvas"
import { Cpu } from "./wasm/cpu"
import { load_module } from "./wasm/wasm"

const module = await load_module('../wasm/wasm32-unknown-unknown/debug/trtl.wasm')

let cpu: Cpu | null = null
let canvas: Canvas | null = null
let sprite: ImageData | null = null

export interface RenderCall {
    input: string,
    width: number,
    height: number
}

const render = () => {
    if (cpu == null || canvas == null || sprite == null) {
        return;
    }

    let startTime, endTime;
    startTime = performance.now()

    cpu.run(canvas, 10_000)
    
    endTime = performance.now()

    console.log(`Call to cpu.run took ${endTime - startTime} milliseconds`)

    
    startTime = performance.now()

    const bytes = canvas.get_bytes()
    
    endTime = performance.now()

    console.log(`Call to canvas.get_bytes took ${endTime - startTime} milliseconds`)


    if (bytes) {
        
        startTime = performance.now()

        sprite.data.set(bytes)
        
        endTime = performance.now()

        console.log(`Call to sprite.data.set took ${endTime - startTime} milliseconds`)

        postMessage(sprite)
    }

    requestAnimationFrame(render)
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
    sprite = new ImageData(width, height)

    render()
}
