export function logRender(message: string) {
    console.log(`%c[RENDERING] ${message}`, 'color: #90EE90')
}

export function logDeletion(message: string) {
    console.log(`%c${message}`, 'color: #FF7F7F')
}

export function logError(message: string) {
    console.error(`%c${message}`, 'color: red')
}