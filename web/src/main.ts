import './style.css'
import { editor, setContent } from './editor/editor';
import { loadModule } from './wasm/wasm';

const module = await loadModule('wasm/wasm32-unknown-unknown/debug/trtl.wasm')
const element = document.querySelector<HTMLDivElement>("div#editor");

if (element && module) {
    const e = editor(element, module)

    const content = ["if (2) { ", "    x = x + 3; ", "} else {", "    x = 2;", "}", "", "func test(x,y) {", "    x = 2;", "}"]

    setContent(e, ...content)

    element.addEventListener('keyup', (e) => {
        
    })
}
