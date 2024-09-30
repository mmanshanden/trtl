import './style.css'
import { editor } from './editor/editor';

const element = document.querySelector<HTMLDivElement>("div#editor");

if (element) editor(element)
