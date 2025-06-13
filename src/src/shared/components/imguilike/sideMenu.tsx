import React, {Dispatch, RefObject, SetStateAction, useEffect, useImperativeHandle, useState} from "react";
import "./sidemenu.scss"
import {clsx} from "clsx";
import {IconProps} from "../icon.js";

export type SidemenuTab = {
    icon: React.ReactElement<IconProps>,
    content: React.JSX.Element | React.JSX.Element[]
}

export type SidemenuProps = {
    ref: RefObject<SideMenuRef>
    onStateChange?: (state: boolean) => void
}

export type SideMenuRef = {
    registerTab: (tag: string, tab: SidemenuTab) => void
    removeTab: (tag: string) => void
    collapse: () => void
    expand: () => void
}

export function SideMenu(props: SidemenuProps) {
    const [selectedMenu, setSelectedMenu] = useState<number>(null)
    const [isCollapsed, setIsCollapsed] = useState<boolean>(true)
    const [tabs, setTabs] = useState<{ [tag: string]: SidemenuTab }>({})

    function onMenuSelect(i: number) {
        if (i === selectedMenu) {
            setSelectedMenu(null)
            setIsCollapsed(true)
            if (props.onStateChange) props.onStateChange(false)
        } else {
            setSelectedMenu(i)
            setIsCollapsed(false)
            if (props.onStateChange) props.onStateChange(true)
        }
    }

    useEffect(() => {
        if (isCollapsed) setSelectedMenu(null)
    }, [isCollapsed]);

    useImperativeHandle(
        props.ref,
        () => {
            return {
                registerTab: (tag, tab) => {
                    setTabs({
                        ...tabs,
                        [tag]: tab
                    })
                },
                removeTab: (tag) => {
                    const newTabs = {...tabs}
                    delete newTabs[tag]
                    setTabs(newTabs)
                },
                collapse: () => {
                    setIsCollapsed(true)
                },
                expand: () => {
                    setIsCollapsed(false)
                },
            }
        },
        [tabs, isCollapsed]
    )

    return (
        <div className={"sidemenu"}>
            <div className={"side-buttons"}>
                {
                    Object.keys(tabs).map((tag, i) => {
                        const t = tabs[tag]

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
                        ? tabs[selectedMenu] ? tabs[selectedMenu].content : <></>
                        : <></>
                }
            </div>
        </div>
    )
}