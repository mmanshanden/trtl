interface Fragment {
    value: string
    style?: 'keyword' | 'identifier'
}

interface Line {
    fragments: Array<Fragment>
}



export const highlight = (input: string[]): Line[] => {
    let lines: Line[] = input.map(line => {
        return { 
            fragments: line.split(/(\s)/).filter(Boolean).map(word => {
                if (["func", "if", "else"].includes(word)) {
                    return { value: word, style: 'keyword' }
                } else {
                    return { value: word } 
                }
            })
        }
    })

    return lines
}