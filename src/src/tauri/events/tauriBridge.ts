import {listen, UnlistenFn} from '@tauri-apps/api/event';
import {invoke} from "@tauri-apps/api/core";
import {BackendResponse, BackendResponseType, TauriCommand, TauriCommandMap, TauriEventMap} from './types.ts';

class TauriBridge {
    private listeners: Map<string, UnlistenFn[]> = new Map();

    async listen<K extends keyof TauriEventMap>(
        event: K,
        callback: (data: TauriEventMap[K]) => void
    ): Promise<UnlistenFn> {
        const unlisten = await listen(event, (event) => {
            callback(event.payload as TauriEventMap[K]);
        });

        const currentListeners = this.listeners.get(event) || [];
        this.listeners.set(event, [...currentListeners, unlisten]);

        return unlisten;
    }

    async invoke<R, E>(
        command: TauriCommand,
        args: TauriCommandMap[typeof command]
    ): Promise<BackendResponse<R, E>> {
        try {
            console.log("%c[TAURI] [INVOKE] Invoking command: " + command, 'color: #FFD580')
            return {
                type: BackendResponseType.Success,
                data: await invoke(command, args)
            };
        } catch (error) {
            console.error("%c[TAURI] [INVOKE] Error invoking command: " + command, 'color: #FFD580')
            return {
                type: BackendResponseType.Error,
                error
            };
        }
    }

    async cleanup(event?: string) {
        if (event) {
            const listeners = this.listeners.get(event) || [];
            await Promise.all(listeners.map(unlisten => unlisten()));
            this.listeners.delete(event);
        } else {
            for (const [_, listeners] of this.listeners) {
                await Promise.all(listeners.map(unlisten => unlisten()));
            }
            this.listeners.clear();
        }
    }
}

export const tauriBridge = new TauriBridge();
