import React from "react";
import {getCurrentWindow} from "@tauri-apps/api/window";
import "./header.scss"
import {useTheme} from "../hooks/useTheme.tsx";
import Icon, {IconName} from "./icon.tsx";

const decorations = {
    minimize: `${process.env.PUBLIC_URL}/icons/decorations/minimize.svg`,
    fullscreen: `${process.env.PUBLIC_URL}/icons/decorations/fullscreen.svg`,
    borderedWindow: `${process.env.PUBLIC_URL}/icons/decorations/bordered-window.svg`,
    close: `${process.env.PUBLIC_URL}/icons/decorations/close.svg`
}

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
                        <Icon name={IconName.Minimize}/>
                    </div>
                    <div className="native-window-control" id="maximize" onClick={async () => {
                        await window.maximize()
                    }}>
                        <Icon name={IconName.Fullscreen}/>
                    </div>
                    <div className="native-window-control" id="close" onClick={() => window.close()}>
                        <Icon name={IconName.Close}/>
                    </div>

                </div>
            </div>
            <div className={`bottom-header`}>
                <div>
                    <button>
                        <Icon name={IconName.FloppyDisk} width={14} height={14}/>
                    </button>
                    <button>
                        <Icon name={IconName.UploadFile} width={14} height={14}/>
                    </button>
                    <button>
                        <Icon name={IconName.DownloadFile} width={14} height={14}/>
                    </button>
                    <button>
                        <Icon name={IconName.NewFolder} width={14} height={14}/>
                    </button>
                    <button className={"delete-button"}>
                        <Icon name={IconName.RecycleBin} width={14} height={14}/>
                    </button>
                </div>
                <div>

                </div>
                <div>
                    <button>
                        <Icon name={IconName.EyeOpen} width={14} height={14}/>
                    </button>
                    <button>
                        <Icon name={IconName.Variation} width={14} height={14}/>
                    </button>
                    <button>
                        <Icon name={IconName.Edit} width={14} height={14}/>
                    </button>
                    <button>
                        <Icon name={IconName.Cog} width={14} height={14}/>
                    </button>
                </div>
            </div>
        </div>
    )
}