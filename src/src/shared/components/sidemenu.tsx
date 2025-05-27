import React, {useEffect, useState} from "react";
import {IconProps} from "./icon.js";
import "./sidemenu.scss"
import {clsx} from "clsx";

export type SidemenuTab = {
    icon: React.ReactElement<IconProps>,
    content: React.JSX.Element | React.JSX.Element[]
}

export type SidemenuProps = {
    tabs: SidemenuTab[]
    setIsCollapsed: React.SetStateAction<boolean>
    isCollapsed: boolean
}

export function Sidemenu(props: SidemenuProps) {
    const [selectedMenu, setSelectedMenu] = useState<number>(null)

    function onMenuSelect(i: number) {
        if (i === selectedMenu) {
            setSelectedMenu(null)
            props.setIsCollapsed(true)
        } else {
            setSelectedMenu(i)
            props.setIsCollapsed(false)
        }
    }

    useEffect(() => {
        if (props.isCollapsed) {
            setSelectedMenu(null)
        }
    }, [props.isCollapsed]);

    return (
        <div className={"sidemenu"}>
            <div className={"side-buttons"}>
                {
                    props.tabs.map((t, i) => {
                        return (
                            <button className={clsx(selectedMenu === i && "selected")}
                                    onClick={() => onMenuSelect(i)} key={`side-button-${i}`}>
                                {t.icon}
                            </button>
                        )
                    })
                }
            </div>
            <div className={"side-content"}>
                {
                    selectedMenu !== null
                        ? props.tabs[selectedMenu] ? props.tabs[selectedMenu].content : <></>
                        : <></>
                }
            </div>
        </div>
    )
}