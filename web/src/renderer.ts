import { Cpu, cpuExec, createCpu, destroyCpu } from "./wasm/cpu"
import { loadModule } from "./wasm/wasm"

const module = await loadModule('../wasm/wasm32-unknown-unknown/debug/trtl.wasm')

let cpu: Cpu | null = null

onmessage = async (e) => {
    if (cpu != null) {
        console.log("destroying cpu at", cpu.ptr)
        destroyCpu(cpu)
    }

    cpu = createCpu(module, e.data)
    console.log("new cpu at", cpu?.ptr)

    if (cpu) {
        const canvas = cpuExec(cpu, 100000);
        const image = new ImageData(new Uint8ClampedArray(canvas), 512, 512)
        
        postMessage(image)
    }
}
