import './style.css'
import { Editor, editor, getContent, setContent } from './editor/editor';
import { load_module } from './wasm/wasm';


const module = await load_module('wasm/wasm32-unknown-unknown/debug/trtl.wasm')
const editorElement = document.querySelector<HTMLDivElement>("div#editor")!;
const canvasElement = document.querySelector<HTMLCanvasElement>("canvas#canvas")!;

canvasElement.width = canvasElement.parentElement?.clientWidth ?? 512;
canvasElement.height = canvasElement.parentElement?.clientHeight ?? 512;

const renderer = new Worker(new URL('./renderer.ts', import.meta.url), {
    type: 'module'
});

const runCpu = (editor: Editor) => {
    const input = getContent(editor).join('\n')
    renderer.postMessage({
        input,
        width: canvasElement.width,
        height: canvasElement.height
    })
}

if (editorElement && module) {
    const e = editor(editorElement, module, {
        contentChanged: runCpu
    });

    const content = [
        "func min(a, b) {",
        "    if a < b {",
        "        return a;",
        "    }",
        "",
        "    return b;",
        "}",
        "",
        "func segment(n, length) {",
        "    if n == 0 {",
        "        forward length;",
        "        return;",
        "    }",
        "",
        "    l = (length / 3);",
        "",
        "    segment(n - 1, l);",
        "    left 60;",
        "    segment(n - 1, l);",
        "    right 120;",
        "    segment(n - 1, l);",
        "    left 60;",
        "    segment(n - 1, l);",
        "}",
        "",
        "func triangle(n, length) {",
        "    left 60;",
        "    segment(n, length);",
        "    right 120;",
        "    segment(n, length);",
        "    right 120;",
        "    segment(n, length);",
        "    left 180;",
        "}",
        "",
        "i = min(0, 5);",
        "",
        "while i < 5 {",
        "    triangle(i, 400);",
        "    i = i + 1;",
        "}",
        ""
    ]

    setContent(e, ...content)
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

window.addEventListener("resize", (e) => {
    canvasElement.width = canvasElement.parentElement?.clientWidth ?? 512;
    canvasElement.height = canvasElement.parentElement?.clientHeight ?? 512;
})
