import React, {useState} from "react"
import "./multimenu.scss"

export type MultiMenuTab = {
    name: string
    content: React.JSX.Element | React.JSX.Element[]
}

export type MultiMenuProps = {
    tabs: MultiMenuTab[]
}

export default function MultiMenu(props: MultiMenuProps) {
    const [selectedTab, setSelectedTab] = useState<number>(0)

    return (
        <div className={"multimenu"}>
            <div className={"tabs-container"}>
                <div className={"tabs"}>
                    {
                        props.tabs.map((t, i) => (
                            <div key={t.name} className={selectedTab === i ? "selected" : ""} onClick={() => setSelectedTab(i)}>
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