interface KeyState {
    key: string  
    pressed: boolean
    control: boolean
}

export interface KeyMap {
    state: Map<string, KeyState>
}

export const pressKey = ({state}: KeyMap, e: KeyboardEvent) => {
    state.set(e.key, {
        key: e.key,
        pressed: true,
        control: e.ctrlKey
    })
}

export const releaseKey = ({state}: KeyMap, e: KeyboardEvent): KeyState => {
    const keyState = state.get(e.key)
    state.set(e.key, {
        key: e.key,
        pressed: false,
        control: false
    })
    
    if (keyState) return keyState
    return state.get(e.key)!
}

