import { Editor } from "./editor";

export interface KeyEvent {
    element: HTMLElement,
    key: KeyboardEvent["key"],
    control: boolean
}

export interface ContentChangedEvent {
    editor: Editor,
    content: string[]
}

interface Events {
    "keyPress": KeyEvent,
    "contentChanged": ContentChangedEvent
}

type Listener<T extends keyof Events> = (e: Events[T]) => void;

type Listeners = {
    [T in keyof Events]: Listener<T>[];
};

export class EventBus {
    listeners: Listeners;

    constructor() {
        this.listeners = {
            'keyPress': [],
            'contentChanged': []
        }
    }

    publish<T extends keyof Events>(type: T, event: Events[T]) {
        this.listeners[type].forEach(listener => listener(event))
    }

    subscribe<T extends keyof Events>(type: T, listener: Listener<T>) {
        this.listeners[type].push(listener)
    }
}