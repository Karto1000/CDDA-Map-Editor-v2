import {Vector2, Vector3} from "three";
import {invoke, InvokeArgs} from "@tauri-apps/api/core";

export function serializedVec2ToVector2(serializedVec2: string): Vector2 {
    const parts = serializedVec2.split(",")

    const x = parseInt(parts[0])
    const y = parseInt(parts[1])

    return new Vector2(x, y)
}

export function serializedVec3ToVector3(serializedVec3: string): Vector3 {
    const parts = serializedVec3.split(",")

    const x = parseInt(parts[0])
    const y = parseInt(parts[1])
    const z = parseInt(parts[2])

    return new Vector3(x, y, z)
}

export enum BackendResponseType {
    Error,
    Success
}

export type BackendResponse<T, E> = {
    type: BackendResponseType.Error,
    error: E
} | {
    type: BackendResponseType.Success,
    data: T
}

export async function invokeTauri<T, E>(command: string, args: InvokeArgs): Promise<BackendResponse<T, E>> {
    try {
        return {
            type: BackendResponseType.Success,
            data: await invoke<T>(command, args),
        }
    } catch (e) {
        return {
            type: BackendResponseType.Error,
            error: e
        }
    }
}