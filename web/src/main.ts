import './style.css'
import { Editor, editor, getContent, setContent } from './editor/editor';
import { loadModule } from './wasm/wasm';

const renderer = new Worker(new URL('./renderer.ts', import.meta.url), {
    type: 'module'
});

const runCpu = (editor: Editor) => {
    const input = getContent(editor).join('\n')
    renderer.postMessage(input)
}

const module = await loadModule('wasm/wasm32-unknown-unknown/debug/trtl.wasm')
const editorElement = document.querySelector<HTMLDivElement>("div#editor");
const canvasElement = document.querySelector<HTMLCanvasElement>("canvas#canvas");

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
        "func straight(n, length) {",
        "    if n == 0 {",
        "        forward length;",
        "        return;",
        "    }",
        "",
        "    l = (length / 3);",
        "",
        "    straight(n - 1, l);",
        "    left 60;",
        "    straight(n - 1, l);",
        "    right 120;",
        "    straight(n - 1, l);",
        "    left 60;",
        "    straight(n - 1, l);",
        "}",
        "",
        "func triangle(n, length) {",
        "    left 60;",
        "    straight(n, length);",
        "    right 120;",
        "    straight(n, length);",
        "    right 120;",
        "    straight(n, length);",
        "    left 180;",
        "}",
        "",
        "i = 5;",
        "",
        "while 1 > 0 {",
        "    triangle(i, 300);",
        "}"
    ]

    setContent(e, ...content)
}


renderer.addEventListener('message', (e) => {
    if (!canvasElement) {
        return
    }

    const image = e.data as ImageData
    console.log("rendering to canvas", image)
    const context = canvasElement.getContext("2d")
    context?.putImageData(image, 0, 0);
})
