import {useEffect} from "react";

export type UseKeybindingsRet = {}

export type KeyListener = {
    key: string
    withAlt?: boolean
    withShift?: boolean
    withCtrl?: boolean
    action: () => void
}


export function useKeybindings(ctx: HTMLElement | Window, keybindings: KeyListener[], deps: any[] = []): UseKeybindingsRet {
    useEffect(() => {
        const localListeners = []

        keybindings.forEach(k => {
            const fn = (e: KeyboardEvent) => {
                if (e.key !== k.key) return;

                if (k.withAlt && !e.altKey) return;
                if (k.withShift && !e.shiftKey) return;
                if (k.withCtrl && !e.ctrlKey) return;

                k.action()
            }

            localListeners.push(fn)

            ctx.addEventListener("keydown", fn)
        })

        return () => {
            localListeners.forEach(l => ctx.removeEventListener("keydown", l))
        }
    }, [...deps, ctx]);

    return {}
}