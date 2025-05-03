import { Caret } from "./editor";

interface State {
    caret: Caret | null
    input_html: string
    gutter_html: string
}

export class Undo {
    stack: State[]
    len: number
    pos: number

    constructor() {
        this.stack = []
        this.len = 0
        this.pos = -1
    }

    push(state: State) {
        this.pos = this.pos + 1
        this.len = this.pos + 1
        this.stack[this.pos] = state
        console.log('push', this.pos, this.len, this.stack)
    }

    undo(): State | null {
        if (this.pos < 0) {
            return null
        }
        
        this.pos = this.pos - 1

        return this.stack[this.pos]
    }

    redo(): State | null {
        console.log(this.pos, this.len)

        if (this.pos >= this.len - 1) {
            return null
        }

        this.pos = this.pos + 1

        return this.stack[this.pos]
    }
}