import React from "react";
import {getCurrentWindow} from "@tauri-apps/api/window";
import "./header.scss"
import {useTheme} from "../hooks/useTheme.tsx";
import Icon, {IconName} from "./icon.tsx";

export function Header() {
    const window = getCurrentWindow();

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
                        await window.minimize()
                    }}>
                        <Icon name={IconName.HideSmall} width={14} height={14} />
                    </div>
                    <div className="native-window-control" id="maximize" onClick={async () => {
                        await window.maximize()
                    }}>
                        <Icon name={IconName.WindowedSmall} width={14} height={14} />
                    </div>
                    <div className="native-window-control" id="close" onClick={() => window.close()}>
                        <Icon name={IconName.CloseSmall} width={14} height={14} />
                    </div>

                </div>
            </div>
            <div className={`bottom-header`}>
                <div>
                    <button>
                        <Icon name={IconName.SaveSmall} width={14} height={14}/>
                    </button>
                    <button>
                        <Icon name={IconName.ExportSmall} width={14} height={14}/>
                    </button>
                    <button>
                        <Icon name={IconName.ImportSmall} width={14} height={14}/>
                    </button>
                    <button>
                        <Icon name={IconName.OpenSmall} width={14} height={14}/>
                    </button>
                    <button className={"delete-button"}>
                        <Icon name={IconName.DeleteSmall} width={14} height={14}/>
                    </button>
                </div>
                <div>

                </div>
                <div>
                    <button>
                        <Icon name={IconName.SettingsSmall} width={14} height={14}/>
                    </button>
                </div>
            </div>
        </div>
    )
}