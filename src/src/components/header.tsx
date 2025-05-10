import React, {Dispatch, MutableRefObject, SetStateAction, useContext} from "react";
import {getCurrentWindow} from "@tauri-apps/api/window";
import "./header.scss"
import Icon, {IconName} from "./icon.tsx";
import {Dropdown} from "./dropdown.tsx";
import {DropdownGroup} from "./dropdown-group.tsx";
import {open} from "@tauri-apps/plugin-shell";
import {TabContext, ThemeContext} from "../app.tsx";
import {invoke} from "@tauri-apps/api/core";

import {MapDataSendCommand} from "../lib/map_data.ts";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {Theme} from "../hooks/useTheme.js";
import {emitTo} from "@tauri-apps/api/event";

type Props = {
    openMapWindowRef: MutableRefObject<WebviewWindow>
    settingsWindowRef: MutableRefObject<WebviewWindow>
}

export function Header(props: Props) {
    const tauriWindow = getCurrentWindow();
    const tabs = useContext(TabContext)
    const {theme, setTheme} = useContext(ThemeContext)

    function onTabClose(e: React.MouseEvent<HTMLDivElement>, index: number) {
        e.preventDefault()
        e.stopPropagation()

        tabs.removeTab(index)
    }

    function onTabCreate() {
    }

    async function onTabOpen(index: number) {
        if (tabs.openedTab === index) {
            tabs.setOpenedTab(null)
            await invoke(MapDataSendCommand.CloseProject, {})
        } else {
            tabs.setOpenedTab(index)
            await invoke(MapDataSendCommand.OpenProject, {index})
        }
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
                                tabs.tabs.map((t, i) => (
                                    <div className={`tab ${tabs.openedTab === i ? "opened-tab" : ""}`} key={i}
                                         onClick={() => onTabOpen(i)}>
                                        <p>{t.name}</p>
                                        <div onClick={e => onTabClose(e, i)}>
                                            <Icon name={IconName.CloseSmall} width={12} height={12}/>
                                        </div>
                                    </div>
                                ))
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
                        <div className="native-window-control" id="close" onClick={() => tauriWindow.close()}>
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
                                        props.settingsWindowRef.current = new WebviewWindow('settings', {
                                            url: `src/windows/settings/window.html?theme=${theme.toString()}`,
                                            width: 200,
                                            height: 200,
                                            decorations: false,
                                            center: true
                                        });

                                        props.settingsWindowRef.current.listen("change-theme", () => {
                                            setTheme((theme) => theme === Theme.Dark ? Theme.Light : Theme.Dark)
                                        })

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