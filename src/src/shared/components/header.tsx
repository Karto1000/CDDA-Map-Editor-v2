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
import {useKeybindings} from "../hooks/useKeybindings.js";

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

    function onAboutClicked() {
        props.aboutWindowRef.current = openWindow(WindowLabel.About, theme)
    }

    function onNewClicked() {
        props.newMapWindowRef.current = openWindow(WindowLabel.NewMap, theme)
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

    function onImport() {
        props.importMapWindowRef.current = openWindow(WindowLabel.ImportMap, theme)
    }

    function onExport() {
        alert("Not Implemented")
    }

    function onSettingsOpen() {
        setSettingsWindow(openWindow(WindowLabel.Settings, theme))
    }

    useKeybindings(
        window,
        [
            {
                key: "n",
                withCtrl: true,
                action: onNewClicked
            },
            {
                key: "o",
                withCtrl: true,
                action: onOpen
            },
            {
                key: "s",
                withCtrl: true,
                action: onSave,
            },
            {
                key: "w",
                withCtrl: true,
                action: onClose
            },
            {
                key: "w",
                withCtrl: true,
                withAlt: true,
                action: onCloseAll
            },
            {
                key: "i",
                withCtrl: true,
                action: onImport
            },
            {
                key: "e",
                withCtrl: true,
                action: onExport
            },
            {
                key: "s",
                withCtrl: true,
                withAlt: true,
                action: onSettingsOpen
            }
        ],
        [tabs]
    )

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

    function onTabCreate() {
        props.importMapWindowRef.current = openWindow(WindowLabel.ImportMap, theme)
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
                                    shortcut: "Ctrl+n",
                                    onClick: (ref) => {
                                        onNewClicked()
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "Open",
                                    shortcut: "Ctrl+o",
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
                                            editorData.recent_projects.map(p => {
                                                return {
                                                    name: p.name,
                                                    onClick: async (ref) => {
                                                        ref.current.closeMenu()
                                                        await onRecentProjectOpen(p.name)
                                                        await onTabOpen(p.name)
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
                                    shortcut: "Ctrl+s",
                                    onClick: (ref) => {
                                        onSave()
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "Close",
                                    shortcut: "Ctr+w",
                                    onClick: async (ref) => {
                                        await onClose()
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "Close All",
                                    shortcut: "Ctr+Alt+w",
                                    onClick: async (ref) => {
                                        await onCloseAll()
                                        ref.current.closeMenu()
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Import",
                                    shortcut: "Ctrl+i",
                                    onClick: (ref) => {
                                        onImport()
                                        ref.current.closeMenu()
                                    }
                                },
                                {
                                    name: "Export",
                                    shortcut: "Ctrl+e",
                                    onClick: (ref) => {
                                        onExport()
                                        ref.current.closeMenu()
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Settings",
                                    shortcut: "Ctrl+Alt+s",
                                    onClick: (ref) => {
                                        onSettingsOpen()
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
                                    shortcut: "Ctrl+z",
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                },
                                {
                                    name: "Redo",
                                    shortcut: "Ctrl+y",
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Copy",
                                    shortcut: "Ctr+c",
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                },
                                {
                                    name: "Paste",
                                    shortcut: "Ctr+v",
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
                                    shortcut: "d",
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                },
                                {
                                    name: "Fill",
                                    shortcut: "f",
                                    onClick: () => {
                                        alert("Not Implemented")
                                    }
                                },
                                {
                                    name: "Erase",
                                    shortcut: "e",
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
                                    onClick: (ref) => {
                                        onAboutClicked()
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