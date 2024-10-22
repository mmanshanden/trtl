import './style.css'
import { editor, setContent } from './editor/editor';
import { loadModule } from './wasm/wasm';

const module = await loadModule('wasm/wasm32-unknown-unknown/debug/trtl.wasm')
const element = document.querySelector<HTMLDivElement>("div#editor");

if (element && module) {
    const e = editor(element, module)

    const content = ["func min(a, b) {", "    if a < b {", "        return a;", "    }", "", "    return b;", "}", "", "func straight(n, length) {", "    if n == 0 {", "        forward length;", "        return;", "    }", "", "    l = (length / 3);", "", "    straight(n - 1, l);", "    left 60;", "    straight(n - 1, l);", "    right 120;", "    straight(n - 1, l);", "    left 60;", "    straight(n - 1, l);", "}", "", "func triangle(n, length) {", "    left 60;", "    straight(n, length);", "    right 120;", "    straight(n, length);", "    right 120;", "    straight(n, length);", "    left 180;", "}", "", "i = 5;", "", "while 1 > 0 {", "    triangle(i, 300);", "}", ]

    setContent(e, ...content)

    element.addEventListener('keyup', (e) => {
        
    })
}
