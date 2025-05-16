import React, {MutableRefObject, useContext, useEffect} from "react";
import {getAllWindows, getCurrentWindow} from "@tauri-apps/api/window";
import "./header.scss"
import Icon, {IconName} from "./icon.tsx";
import {Dropdown} from "./dropdown.tsx";
import {DropdownGroup} from "./dropdown-group.tsx";
import {open} from "@tauri-apps/plugin-shell";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {TabContext, ThemeContext} from "../../app.js";
import {ChangedThemeEvent, CloseLocalTabEvent, LocalEvent, OpenLocalTabEvent} from "../utils/localEvent.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {TauriCommand} from "../../tauri/events/types.js";
import {openWindow, WindowLabel} from "../../windows/lib.js";
import {Theme} from "../hooks/useTheme.js";

type Props = {
    eventBus: MutableRefObject<EventTarget>

    openMapWindowRef: MutableRefObject<WebviewWindow>
    settingsWindowRef: MutableRefObject<WebviewWindow>
}

export function Header(props: Props) {
    const tauriWindow = getCurrentWindow();
    const {theme} = useContext(ThemeContext)
    const tabs = useContext(TabContext)
    const [settingsWindow, setSettingsWindow] = React.useState<WebviewWindow | null>(null)

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

    async function onTabClose(e: React.MouseEvent<HTMLDivElement>, name: string) {
        console.log(`Closed tab ${name}`)

        e.preventDefault()
        e.stopPropagation()

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
        props.openMapWindowRef.current = openWindow(WindowLabel.OpenMap, theme)
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
            await tauriBridge.invoke(
                TauriCommand.OPEN_PROJECT,
                {
                    name: name
                }
            )

            props.eventBus.current.dispatchEvent(
                new OpenLocalTabEvent(
                    LocalEvent.OPEN_LOCAL_TAB,
                    {detail: {name: name}}
                )
            )
        }
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
                                        <p>{t.name}</p>
                                        <div onClick={e => onTabClose(e, t.name)}>
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
                                    onClick: () => {
                                    }
                                },
                                {
                                    name: "Open",
                                    shortcut: "Ctrl+o",
                                    onClick: () => {
                                    }
                                },
                                {
                                    name: "Open Recent",
                                    expandable: true,
                                    onClick: () => {
                                    },
                                    subGroups: [
                                        [
                                            {
                                                name: "house_01",
                                                onClick: () => {
                                                }
                                            }
                                        ]
                                    ]
                                }
                            ],
                            [
                                {
                                    name: "Save",
                                    shortcut: "Ctrl+s",
                                    onClick: () => {
                                    }
                                },
                                {
                                    name: "Close",
                                    shortcut: "Ctr+w",
                                    onClick: () => {
                                    }
                                },
                                {
                                    name: "Close All",
                                    shortcut: "Ctr+Shift+w",
                                    onClick: () => {
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Import",
                                    shortcut: "Ctrl+i",
                                    onClick: () => {
                                    }
                                },
                                {
                                    name: "Export",
                                    shortcut: "Ctrl+e",
                                    onClick: () => {
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Settings",
                                    shortcut: "Ctrl+Alt+s",
                                    onClick: (ref) => {
                                        setSettingsWindow(openWindow(WindowLabel.Settings, theme))
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
                                    }
                                },
                                {
                                    name: "Redo",
                                    shortcut: "Ctrl+y",
                                    onClick: () => {
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Copy",
                                    shortcut: "Ctr+c",
                                    onClick: () => {
                                    }
                                },
                                {
                                    name: "Paste",
                                    shortcut: "Ctr+v",
                                    onClick: () => {
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
                                                }
                                            },
                                            {
                                                name: "Circle",
                                                isToggleable: true,
                                                toggled: false,
                                                onClick: () => {
                                                }
                                            }
                                        ]
                                    ]
                                },
                                {
                                    name: "Draw",
                                    shortcut: "d",
                                    onClick: () => {
                                    }
                                },
                                {
                                    name: "Fill",
                                    shortcut: "f",
                                    onClick: () => {
                                    }
                                },
                                {
                                    name: "Erase",
                                    shortcut: "e",
                                    onClick: () => {
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
                                                }
                                            },
                                            {
                                                name: "Circle",
                                                isToggleable: true,
                                                toggled: false,
                                                onClick: () => {
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
                                    toggled: true,
                                    onClick: () => {
                                    }
                                }
                            ],
                            [
                                {
                                    name: "Tileset",
                                    expandable: true,
                                    onClick: () => {
                                    },
                                    subGroups: [
                                        [
                                            {
                                                name: "UndeadPeopleTileset",
                                                isToggleable: true,
                                                toggled: false,
                                                onClick: () => {
                                                }
                                            }
                                        ],
                                        [
                                            {
                                                name: "Select New",
                                                onClick: () => {
                                                }
                                            }
                                        ]
                                    ]
                                }
                            ]
                        ]}/>
                        <Dropdown name={"Help"} groups={[
                            [
                                {
                                    name: "GitHub",
                                    onClick: async () => {
                                        await open("https://github.com/Karto1000/CDDA-Map-Editor-v2")
                                    }
                                },
                                {
                                    name: "CDDA",
                                    onClick: async () => {
                                        await open("https://github.com/CleverRaven/Cataclysm-DDA");
                                    }
                                }
                            ],
                            [
                                {
                                    name: "About",
                                    onClick: () => {
                                        alert("TBD");
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