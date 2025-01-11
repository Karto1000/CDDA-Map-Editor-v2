import React, {Dispatch, SetStateAction} from "react";
import {getCurrentWindow} from "@tauri-apps/api/window";
import "./header.scss"
import Icon, {IconName} from "./icon.tsx";
import {Dropdown} from "./dropdown.tsx";
import {DropdownGroup} from "./dropdown-group.tsx";
import {open} from "@tauri-apps/plugin-shell";

type Props = {
    isSettingsWindowOpen: boolean,
    setIsSettingsWindowOpen: Dispatch<SetStateAction<boolean>>,
}

export function Header(props: Props) {
    const tauriWindow = getCurrentWindow();

    return (
        <div className={`header-container`}>
            <div data-tauri-drag-region className={`header`}>
                <div className={"header-title"}>
                    <img
                        src={`${process.env.PUBLIC_URL}/icons/icon.ico`}
                        alt={"icon"}
                        width={24}
                        height={24}
                    />
                    <h1>
                        CDDA Map Editor
                    </h1>
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
                                shortcut: "Ctrl+n"
                            },
                            {
                                name: "Open",
                                shortcut: "Ctrl+o",
                            },
                            {
                                name: "Open Recent",
                                expandable: true,
                                subGroups: [
                                    [
                                        {
                                            name: "house_01"
                                        }
                                    ]
                                ]
                            }
                        ],
                        [
                            {
                                name: "Save",
                                shortcut: "Ctrl+s"
                            },
                            {
                                name: "Close",
                                shortcut: "Ctr+w"
                            },
                            {
                                name: "Close All",
                                shortcut: "Ctr+Shift+w"
                            }
                        ],
                        [
                            {
                                name: "Import",
                                shortcut: "Ctrl+i"
                            },
                            {
                                name: "Export",
                                shortcut: "Ctrl+e"
                            }
                        ],
                        [
                            {
                                name: "Settings",
                                shortcut: "Ctrl+Alt+s",
                                onClick: (ref) => {
                                    props.setIsSettingsWindowOpen(!props.isSettingsWindowOpen);
                                    ref.current.closeMenu()
                                }
                            },
                            {
                                name: "Exit"
                            }
                        ]
                    ]}/>
                    <Dropdown name={"Edit"} groups={[
                        [
                            {
                                name: "Undo",
                                shortcut: "Ctrl+z"
                            },
                            {
                                name: "Redo",
                                shortcut: "Ctrl+y"
                            }
                        ],
                        [
                            {
                                name: "Copy",
                                shortcut: "Ctr+c"
                            },
                            {
                                name: "Paste",
                                shortcut: "Ctr+v"
                            }
                        ],
                        [
                            {
                                name: "Select",
                                expandable: true,
                                subGroups: [
                                    [
                                        {
                                            name: "Rectangle",
                                            isToggleable: true,
                                            toggled: false
                                        },
                                        {
                                            name: "Circle",
                                            isToggleable: true,
                                            toggled: false
                                        }
                                    ]
                                ]
                            },
                            {
                                name: "Draw",
                                shortcut: "d"
                            },
                            {
                                name: "Fill",
                                shortcut: "f"
                            },
                            {
                                name: "Erase",
                                shortcut: "e"
                            },
                            {
                                name: "Shape",
                                expandable: true,
                                subGroups: [
                                    [
                                        {
                                            name: "Rectangle",
                                            isToggleable: true,
                                            toggled: false
                                        },
                                        {
                                            name: "Circle",
                                            isToggleable: true,
                                            toggled: false
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
                            }
                        ],
                        [
                            {
                                name: "Tileset",
                                expandable: true,
                                subGroups: [
                                    [
                                        {
                                            name: "UndeadPeopleTileset",
                                            isToggleable: true,
                                            toggled: false,
                                        }
                                    ],
                                    [
                                        {
                                            name: "Select New"
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
    )
}