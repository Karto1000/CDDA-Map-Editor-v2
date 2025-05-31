import React, {useEffect, useState} from "react"
import "./multimenu.scss"
import {clsx} from "clsx";


export type MultiMenuTab = {
    name: string
    content: React.JSX.Element | React.JSX.Element[]
    isDisabled?: boolean
}

export type MultiMenuProps = {
    tabs: MultiMenuTab[]
    onTabSelected?: (tab: MultiMenuTab) => void
}

export function MultiMenu(props: MultiMenuProps) {
    const [selectedTab, setSelectedTab] = useState<number>(props.tabs.findIndex(t => !t.isDisabled))

    return (
        <div className={"multimenu"}>
            <div className={"tabs-container"}>
                <div className={"tabs"}>
                    {
                        props.tabs.map((t, i) => (
                            <div key={t.name}
                                 className={clsx("tab", selectedTab === i && "selected", t.isDisabled && "disabled")}
                                 onClick={() => {
                                     if (t.isDisabled) return;

                                     if (props.onTabSelected) props.onTabSelected(t);
                                     setSelectedTab(i)
                                 }}>
                                {t.name}
                            </div>
                        ))
                    }
                </div>
                <div className={"tab-line"}/>
            </div>
            {
                props.tabs.length > 0 &&
                <div className={"multimenu-body"}>
                    {props.tabs[selectedTab].content}
                </div>
            }
        </div>
    )
}