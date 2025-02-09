import './style.css'
import { Editor } from './editor/editor';
import { loadModule } from './wasm/wasm';
import { EventBus } from './editor/bus';
import { Input } from './editor/input';

const bus = new EventBus()

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
    const module = await loadModule('wasm/wasm32-unknown-unknown/debug/trtl.wasm')
    const editorElement = document.querySelector<HTMLDivElement>("div#editor")!;
    const canvasElement = document.querySelector<HTMLCanvasElement>("div#canvas canvas")!;
    
    fixCanvasDimensions(canvasElement)
    
    if (module) {
        const editor = new Editor(editorElement, module, bus)
    
        const content = [
            "func min(a, b) {",
            "  if a < b {",
            "    return a;",
            "  }",
            "",
            "  return b;",
            "}",
            "",
            "func segment(n, length) {",
            "  if n == 0 {",
            "    forward length;",
            "    return;",
            "  }",
            "",
            "  l = (length / 3);",
            "",
            "  segment(n - 1, l);",
            "  left 60;",
            "  segment(n - 1, l);",
            "  right 120;",
            "  segment(n - 1, l);",
            "  left 60;",
            "  segment(n - 1, l);",
            "}",
            "",
            "func triangle(n, length) {",
            "  left 60;",
            "  segment(n, length);",
            "  right 120;",
            "  segment(n, length);",
            "  right 120;",
            "  segment(n, length);",
            "  left 180;",
            "}",
            "",
            "i = min(3, 5);",
            "",
            "while i < 6 {",
            "  triangle(i, 400);",
            "  i = i + 1;",
            "}",
            ""
        ]

        bus.subscribe('contentChanged', () => {
            renderer.postMessage({
                input: editor.getContent().join('\n'),
                width: canvasElement.width,
                height: canvasElement.height
            })
        })
                
        window.addEventListener("resize", () => {
            fixCanvasDimensions(canvasElement)
            renderer.postMessage({
                input: editor.getContent().join('\n'),
                width: canvasElement.width,
                height: canvasElement.height
            })
        })

        editor.setContent(...content)
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