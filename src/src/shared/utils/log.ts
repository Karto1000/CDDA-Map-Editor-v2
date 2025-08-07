import {emit, } from "@tauri-apps/api/event";
import {TauriEvent, ToastType} from "../../tauri/events/types.js";

export function logRender(message: string) {
    console.log(`%c[RENDERING] ${message}`, 'color: #90EE90')
}

export function logDeletion(message: string) {
    console.log(`%c${message}`, 'color: #FF7F7F')
}

export function logError(message: string) {
    console.error(`%c${message}`, 'color: red')
}

export async function logToastSuccess(message: string) {
    await emit(
        TauriEvent.EMIT_TOAST_MESSAGE,
        {
            type: ToastType.Success,
            message: message
        }
    )
}

export async function logToastError(message: string) {
    await emit(
        TauriEvent.EMIT_TOAST_MESSAGE,
        {
            type: ToastType.Error,
            message: message
        }
    )
}