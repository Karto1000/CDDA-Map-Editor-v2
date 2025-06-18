import React, {RefObject, useContext, useEffect, useState} from "react";
import {getAllWindows, getCurrentWindow} from "@tauri-apps/api/window";
import "./header.scss"
import Icon, {IconName} from "./icon.tsx";
import {Dropdown} from "./dropdown.tsx";
import {DropdownGroup} from "./dropdown-group.tsx";
import {open} from "@tauri-apps/plugin-shell";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {EditorDataContext, TabContext, ThemeContext} from "../../app.js";
import {
    ChangedThemeEvent,
    CloseLocalTabEvent,
    LocalEvent,
    OpenLocalTabEvent,
    ToggleGridEvent
} from "../utils/localEvent.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {TauriCommand} from "../../tauri/events/types.js";
import {openWindow, WindowLabel} from "../../windows/lib.js";
import {Theme} from "../hooks/useTheme.js";
import {TabTypeKind} from "../hooks/useTabs.js";
import {useKeybindActionEvent} from "../hooks/useKeybindings.js";
import {getKeybindingText, KeybindAction} from "../../tauri/types/editor.js";

type Props = {
    eventBus: RefObject<EventTarget>
    importMapWindowRef: RefObject<WebviewWindow>
    newMapWindowRef: RefObject<WebviewWindow>
    settingsWindowRef: RefObject<WebviewWindow>
    aboutWindowRef: RefObject<WebviewWindow>
    showGrid: RefObject<boolean>
}

export function Header(props: Props) {
    const tauriWindow = getCurrentWindow();
    const {theme} = useContext(ThemeContext)
    const tabs = useContext(TabContext)
    const [settingsWindow, setSettingsWindow] = useState<WebviewWindow | null>(null)
    const [showGrid, setShowGrid] = useState<boolean>(true)

    const editorData = useContext(EditorDataContext)

    useKeybindActionEvent(
        KeybindAction.OpenProject,
        onOpen,
        props.eventBus,
        []
    )

    useKeybindActionEvent(
        KeybindAction.NewProject,
        onNewClicked,
        props.eventBus,
        []
    )

    useKeybindActionEvent(
        KeybindAction.SaveProject,
        onSave,
        props.eventBus,
        []
    )

    useKeybindActionEvent(
        KeybindAction.CloseTab,
        onClose,
        props.eventBus,
        [tabs]
    )

    useKeybindActionEvent(
        KeybindAction.CloseAllTabs,
        onCloseAll,
        props.eventBus,
        [tabs]
    )

    useKeybindActionEvent(
        KeybindAction.ImportMap,
        onImport,
        props.eventBus,
        []
    )

    useKeybindActionEvent(
        KeybindAction.ExportMap,
        onExport,
        props.eventBus,
        []
    )

    useKeybindActionEvent(
        KeybindAction.OpenSettings,
        onSettingsOpen,
        props.eventBus,
        []
    )

    // This component is never unmounted, so we don't have to call unlisten
    async function onAboutClicked() {
        props.aboutWindowRef.current = await openWindow(WindowLabel.About, theme)[0]
    }

    async function onNewClicked() {
        props.newMapWindowRef.current = await openWindow(WindowLabel.NewMap, theme, {defaultWidth: 800, defaultHeight: 500})[0]
    }

    async function onImport() {
        props.importMapWindowRef.current = await openWindow(WindowLabel.ImportMap, theme, {
            defaultWidth: 800,
            defaultHeight: 500
        })[0]
    }

    function onOpen() {
        alert("Not Implemented")
    }

    function onSave() {
        alert("Not Implemented")
    }

    async function onClose() {
        if (!tabs.openedTab) return

        await onTabClose(tabs.openedTab)
    }

    async function onCloseAll() {
        for (const tab of Object.keys(tabs.tabs)) {
            await onTabClose(tab)
        }
    }

    function onExport() {
        alert("Not Implemented")
    }

    async function onSettingsOpen() {
        setSettingsWindow(await openWindow(WindowLabel.Settings, theme)[1])
    }

    useEffect(() => {
        props.settingsWindowRef.current = settingsWindow

        if (!settingsWindow) return

        const unlisten = props.settingsWindowRef.current.listen("change-theme", () => {
            props.eventBus.current.dispatchEvent(
                new ChangedThemeEvent(
                    LocalEvent.CHANGE_THEME_REQUEST,
                    {detail: {theme: theme === Theme.Light ? Theme.Dark : Theme.Light}}
                )
            )
        })

        return () => {
            unlisten.then(f => f())
        }
    }, [settingsWindow, theme]);

    async function onTabClose(name: string) {
        console.log(`Closed tab ${name}`)

        props.eventBus.current.dispatchEvent(
            new CloseLocalTabEvent(
                LocalEvent.REMOVE_LOCAL_TAB,
                {detail: {name: name}}
            )
        )

        await tauriBridge.invoke(
            TauriCommand.CLOSE_PROJECT,
            {name}
        )
    }

    async function onTabCreate() {
        props.importMapWindowRef.current = await openWindow(WindowLabel.ImportMap, theme)[1]
    }

    async function onTabOpen(name: string) {
        if (tabs.openedTab === name) {
            props.eventBus.current.dispatchEvent(
                new CloseLocalTabEvent(
                    LocalEvent.CLOSE_LOCAL_TAB,
                    {detail: {name: name}}
                )
            )
        } else {
            props.eventBus.current.dispatchEvent(
                new OpenLocalTabEvent(
                    LocalEvent.OPEN_LOCAL_TAB,
                    {detail: {name: name}}
                )
            )

            await tauriBridge.invoke(
                TauriCommand.OPEN_PROJECT,
                {
                    name: name
                }
            )
        }
    }

    async function onRecentProjectOpen(name: string) {
        await tauriBridge.invoke(TauriCommand.OPEN_RECENT_PROJECT, {name: name})
    }

    async function onWindowClose() {
        const windows = await getAllWindows()
        // We only want to close the other windows.
        // If we close the main window, sometimes the other windows will not
        // close since the code that closes the window is inside the main window
        windows.filter(w => w.label !== "main")

        for (const w of windows) {
            await w.close();
        }

        await tauriWindow.close();
    }

    function getKeyboardShortcutForAction(action: KeybindAction | null) {
        const found = editorData?.config.keybinds.find(kb => kb.action === action)

        if (!found) return null

        return getKeybindingText(found)
    }

    return (
        <header>
            <div className={`header-container`}>
                <div data-tauri-drag-region className={`header`}>
                    <div className={"header-title"}>
                        <img
                            src={`/icons/icon.ico`}
                            alt={"icon"}
                            width={24}
                            height={24}
                        />
                        <h1>
                            CDDA Map Editor
                        </h1>

                        <div className={"tab-container"}>
                            {
                                Object.keys(tabs.tabs).map((tabName, i) => {
                                    const t = tabs.tabs[tabName]

                                    return <div className={`tab ${tabs.openedTab === tabName ? "opened-tab" : ""}`}
                                                key={i}
                                                onClick={() => onTabOpen(t.name)}>
                                        {
                                            t.tab_type === TabTypeKind.LiveViewer &&
                                            <div className={"tab-type-indicator"}>
                                                <Icon name={IconName.EyeMedium} width={16} height={16}/>
                                            </div>
                                        }
                                        {
                                            t.tab_type === TabTypeKind.MapEditor &&
                                            <div className={"tab-type-indicator"}>
                                                <Icon name={IconName.BucketMedium} width={16} height={16}/>
                                            </div>
                                        }
                                        <p>{t.name}</p>
                                        <div onClick={async (e) => {
                                            e.preventDefault()
                                            e.stopPropagation()
                                            await onTabClose(t.name)
                                        }
                                        } className={"close-tab-button"}>
                                            <Icon name={IconName.CloseSmall} width={12} height={12}/>
                                        </div>
                                    </div>
                                })
                            }
                            <button id={"add-new-tab-button"} onClick={onTabCreate}>
                                <Icon name={IconName.AddSmall} width={16} height={16}/>
                            </button>
                        </div>
                    </div>

                    <div className={"window-control"}>
                        <div className="native-window-control" id="minimize" onClick={async () => {
                            await tauriWindow.minimize()
                        }}>
                            <Icon name={IconName.HideSmall} width={14} height={14}/>
                        </div>
                        <div className="native-window-control" id="maximize" onClick={async () => {
                            await tauriWindow.maximize()
                        }}>
                            <Icon name={IconName.WindowedSmall} width={14} height={14}/>
                        </div>
                        <div className="native-window-control" id="close" onClick={onWindowClose}>
                            <Icon name={IconName.CloseSmall} width={14} height={14}/>
                        </div>

                    </div>
                </div>
                <div className={`bottom-header`}>
                    <DropdownGroup>
                        <Dropdown name={"File"} groups={[
                            [
                                {
                                    name: "New",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.NewProject),
                                    onClick: (ref) => {
                                        onNewClicked()
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "Open",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.OpenProject),
                                    onClick: (ref) => {
                                        onOpen()
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "Open Recent",
                                    expandable: true,
                                    onClick: () => {
                                    },
                                    subGroups: [
                                        editorData ?
                                            Object.keys(editorData.recent_projects).map(name => {
                                                return {
                                                    name: name,
                                                    onClick: async (ref) => {
                                                        ref.current.closeMenu()
                                                        await onRecentProjectOpen(name)
                                                        await onTabOpen(name)
                                                    }
                                                }
                                            })
                                            : []
                                    ]
                                }
                            ],
                            [
                                {
                                    name: "Save",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.SaveProject),
                                    onClick: (ref) => {
                                        onSave()
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "Close",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.CloseTab),
                                    onClick: async (ref) => {
                                        await onClose()
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "Close All",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.CloseAllTabs),
                                    onClick: async (ref) => {
                                        await onCloseAll()
                                        ref.current.closeMenu()
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Import",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.ImportMap),
                                    onClick: async (ref) => {
                                        await onImport()
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "Export",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.ExportMap),
                                    onClick: async (ref) => {
                                        onExport()
                                        ref.current.closeMenu()
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Settings",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.OpenSettings),
                                    onClick: async (ref) => {
                                        await onSettingsOpen()
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "Exit",
                                    onClick: async () => {
                                        await tauriWindow.close()
                                    }
                                }
                            ]
                        ]}/>
                        <Dropdown name={"Edit"} groups={[
                            [
                                {
                                    name: "Undo",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.Undo),
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                },
                                {
                                    name: "Redo",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.Redo),
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Copy",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.Copy),
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                },
                                {
                                    name: "Paste",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.Paste),
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Select",
                                    expandable: true,
                                    onClick: () => {
                                    },
                                    subGroups: [
                                        [
                                            {
                                                name: "Rectangle",
                                                isToggleable: true,
                                                toggled: false,
                                                onClick: () => {
                                                    alert("Not Implemented")
                                                }
                                            },
                                            {
                                                name: "Circle",
                                                isToggleable: true,
                                                toggled: false,
                                                onClick: () => {
                                                    alert("Not Implemented")
                                                }
                                            }
                                        ]
                                    ]
                                },
                                {
                                    name: "Draw",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.Draw),
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                },
                                {
                                    name: "Fill",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.Fill),
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                },
                                {
                                    name: "Erase",
                                    shortcut: getKeyboardShortcutForAction(KeybindAction.Erase),
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                },
                                {
                                    name: "Shape",
                                    expandable: true,
                                    onClick: () => {
                                    },
                                    subGroups: [
                                        [
                                            {
                                                name: "Rectangle",
                                                isToggleable: true,
                                                toggled: false,
                                                onClick: () => {
                                                    alert("Not Implemented")
                                                }
                                            },
                                            {
                                                name: "Circle",
                                                isToggleable: true,
                                                toggled: false,
                                                onClick: () => {
                                                    alert("Not Implemented")
                                                }
                                            }
                                        ]
                                    ]
                                }
                            ]
                        ]}/>
                        <Dropdown name={"View"} groups={[
                            [
                                {
                                    name: "Show Grid",
                                    isToggleable: true,
                                    toggled: showGrid,
                                    onClick: (ref) => {
                                        setShowGrid(!showGrid)

                                        props.showGrid.current = !showGrid
                                        props.eventBus.current.dispatchEvent(
                                            new ToggleGridEvent(
                                                LocalEvent.TOGGLE_GRID,
                                                {detail: {state: !showGrid}}
                                            )
                                        )

                                        ref.current.closeMenu()
                                    }
                                }
                            ],
                        ]}/>
                        <Dropdown name={"Help"} groups={[
                            [
                                {
                                    name: "GitHub",
                                    onClick: async (ref) => {
                                        await open("https://github.com/Karto1000/CDDA-Map-Editor-v2")
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "CDDA",
                                    onClick: async (ref) => {
                                        await open("https://github.com/CleverRaven/Cataclysm-DDA");
                                        ref.current.closeMenu()
                                    }
                                }
                            ],
                            [
                                {
                                    name: "About",
                                    onClick: async (ref) => {
                                        await onAboutClicked()
                                        ref.current.closeMenu()
                                    }
                                }
                            ]
                        ]}/>
                    </DropdownGroup>
                </div>
            </div>
        </header>
    )
}