import {TauriEventMap} from "../../tauri/events/types.js";
import {useEffect, useRef} from "react";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";

export function useTauriEvent<K extends keyof TauriEventMap>(
    event: K,
    callback: (data: TauriEventMap[K]) => (() => void) | void,
    deps: any[] = []
) {
    const savedCallback = useRef(callback);
    const savedCleanup = useRef<(() => void) | void>(undefined);

    useEffect(() => {
        savedCallback.current = callback;
    }, [callback]);

    useEffect(() => {
        console.log("%c[TAURI] [EVENT] Subscribing to event: " + event, 'color: #add8e6')

        const unsubscribe = tauriBridge.listen(event, (data) => {
            console.log("%c[TAURI] [EVENT] Received " + event, 'color: #add8e6')
            savedCleanup.current = savedCallback.current(data);
        });

        return () => {
            console.log("%c[TAURI] [EVENT] Unsubscribing from event: " + event, 'color: #FF7F7F')
            unsubscribe.then(f => f())
            if (savedCleanup.current) savedCleanup.current();
        };
    }, [event, ...deps]);
}