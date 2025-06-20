import {RefObject, useEffect} from "react";
import {ProgramData, KeybindAction} from "../../tauri/types/editor.js";
import {emit} from "@tauri-apps/api/event";
import {TauriEvent} from "../../tauri/events/types.js";
import {useTauriEvent} from "./useTauriEvent.js";

export type UseKeybindingsRet = {}

export function useKeybindings(
    ctx: HTMLElement | Window,
    editorData: ProgramData,
    deps: any[] = []
): UseKeybindingsRet {
    useEffect(() => {
        if (!ctx) return;

        function onKeyDown(e: KeyboardEvent) {
            // Sort keybinds by specificity (number of modifiers)
            const sortedKeybinds = [...editorData.config.keybinds].sort((a, b) => {
                const aModifiers = Number(a.withAlt) + Number(a.withCtrl) + Number(a.withShift);
                const bModifiers = Number(b.withAlt) + Number(b.withCtrl) + Number(b.withShift);
                return bModifiers - aModifiers;
            });

            for (const keybinding of sortedKeybinds) {
                if (e.key !== keybinding.key) continue;

                if (keybinding.withAlt && !e.altKey) continue;
                if (keybinding.withShift && !e.shiftKey) continue;
                if (keybinding.withCtrl && !e.ctrlKey) continue;

                if (!keybinding.withAlt && e.altKey) continue;
                if (!keybinding.withShift && e.shiftKey) continue;
                if (!keybinding.withCtrl && e.ctrlKey) continue;

                if (keybinding.isGlobal) e.preventDefault();

                emit(
                    TauriEvent.KEYBIND_PRESSED,
                    keybinding.action
                )
                return
            }
        }

        ctx.addEventListener("keydown", onKeyDown)

        return () => {
            ctx.removeEventListener("keydown", onKeyDown)
        }
    }, [...deps, ctx]);

    return {}
}

export function useKeybindActionEvent(
    event: KeybindAction,
    fun: () => void,
    deps: any[] = []
) {
    useTauriEvent(
        TauriEvent.KEYBIND_PRESSED,
        action => {
            if (event === action) {
                fun()
            }
        },
        deps
    )
}