import './style.css'
import { editor } from './editor/editor';
import { loadModule, read_return_string_from_module } from './wasm/wasm';

const module = await loadModule('wasm/wasm32-unknown-unknown/release/trtl.wasm')
const element = document.querySelector<HTMLDivElement>("div#editor");

if (element) {
    editor(element)

    element.addEventListener('keyup', (e) => {
        const value = element.innerText
        const bytes = new TextEncoder().encode(value)
        const ptr = module.exports.malloc(bytes.length) as number

        new Uint8Array(module.memory.buffer).set(bytes, ptr)
        
        const ret_ptr = module.exports.echo(ptr, bytes.length) as number
        console.log(read_return_string_from_module(module, ret_ptr))
    })
}