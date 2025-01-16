import { EventBus } from "./bus"

interface KeyState {
    controlHeld: boolean
}

export class Input {
    element: HTMLElement
    state: Map<String, KeyState>
    bus: EventBus

    constructor(element: HTMLElement, bus: EventBus) {
        this.element = element
        this.state = new Map()
        this.bus = bus

        element.addEventListener('keydown', (e) => this.keydown(e))
        element.addEventListener('keyup', (e) => this.keyup(e))
    }

    private keydown(e: KeyboardEvent) {
        this.state.set(e.key, {
            controlHeld: e.ctrlKey
        })
    }

    private keyup(e: KeyboardEvent) {
        let keyState = this.state.get(e.key)

        if (keyState) {
            this.state.delete(e.key)
        }

        this.bus.publish('keyPress', {
            key: e.key,
            control: keyState ? keyState.controlHeld : false,
            element: this.element
        })
    }
}
