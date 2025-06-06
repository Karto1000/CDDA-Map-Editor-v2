import {TauriEventMap} from "../../tauri/events/types.js";
import {useEffect, useRef} from "react";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";

export function useTauriEvent<K extends keyof TauriEventMap>(
    event: K,
    callback: (data: TauriEventMap[K]) => void,
    deps: any[] = []
) {
    const savedCallback = useRef(callback);

    useEffect(() => {
        savedCallback.current = callback;
    }, [callback]);

    useEffect(() => {
        console.log("Subscribing to event: ", event)

        const unsubscribe = tauriBridge.listen(event, (data) => {
            console.log("Received event: ", event)
            savedCallback.current(data);
        });

        return () => {
            console.log("Unsubscribing from event: ", event)
            unsubscribe.then(f => f())
        };
    }, [event, ...deps]);
}
