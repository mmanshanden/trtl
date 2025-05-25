import './style.css'
import { loadModule } from './wasm/wasm';
import { shapes } from './shapes';
import { Editor } from './editor/editor';


const loadRenderer = (): Promise<Worker> => {
    return new Promise(resolve => {
        const renderer = new Worker(new URL('./renderer.ts', import.meta.url), {
            type: 'module'
        });

        const listen = (e: MessageEvent) => {
            if (e.data === 1) {
                renderer.removeEventListener('message', listen)
                resolve(renderer)
            }

            renderer.removeEventListener('message', listen)
        }

        renderer.addEventListener('message', listen)
    })
}

const init = async () => {
    const renderer = await loadRenderer()
    const module = await loadModule('wasm/wasm32-unknown-unknown/release/trtl.wasm')
    const editorElement = document.querySelector<HTMLDivElement>("div#editor")!;
    const canvasElement = document.querySelector<HTMLCanvasElement>("canvas#canvas")!;
    
    fixCanvasDimensions(canvasElement)
    
    if (module) {
        const editor = new Editor(editorElement, module)

        editor.addEventListener('render', () => {
            renderer.postMessage({
                input: editor.getContent(),
                width: canvasElement.width,
                height: canvasElement.height
            })
        })
                
        window.addEventListener("resize", () => {
            fixCanvasDimensions(canvasElement)
            renderer.postMessage({
                input: editor.getContent(),
                width: canvasElement.width,
                height: canvasElement.height
            })
        })

        const fileSelector = document.querySelector<HTMLSelectElement>('select')!

        fileSelector.addEventListener('change', (e) => {
            const target = e.target as HTMLSelectElement
            const value = target.value

            if (value === 'koch') {
                editor.setContent(shapes.koch.join("\n"))
            } else if (value === 'spiral') {
                editor.setContent(shapes.spiral.join("\n"))
            } else if (value === 'emoji_test') {
                editor.setContent(shapes.emoji_test.join("\n"))
            }
        })

        editor.focus()
        editor.setContent("")
    }
    
    
    renderer.addEventListener('message', (e) => {
        if (!canvasElement) {
            return
        }
    
        const image = e.data as ImageData
        const context = canvasElement.getContext("2d")
    
        if (context) {
            context.putImageData(image, 0, 0);
        }
    })

}

window.addEventListener('DOMContentLoaded', async () => {
    await init()
})


const fixCanvasDimensions = (canvas: HTMLCanvasElement) => {
    const parent = canvas.parentElement
    
    if (parent) {
        canvas.width = parent.clientWidth ?? 512
        canvas.height = parent.clientHeight ?? 512
    }
}