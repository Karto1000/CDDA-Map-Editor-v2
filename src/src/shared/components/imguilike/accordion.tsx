import React, {useState} from "react";
import {ReactNode} from "react";
import "./accordion.scss"
import Icon, {IconName} from "../icon.js";

export type AccordionProps = {
    title: string,
    defaultCollapsed?: boolean,
    children: ReactNode[] | ReactNode,
}

export function Accordion(props: AccordionProps) {
    const [isCollapsed, setIsCollapsed] = useState<boolean>(props.defaultCollapsed ? props.defaultCollapsed : false)

    return (
        <div className={"accordion"}>
            <button className={"accordion-header"} onClick={() => setIsCollapsed(!isCollapsed)}>
                <h2>{props.title}</h2>
                <Icon name={isCollapsed ? IconName.ChevronDownSmall : IconName.ChevronUpSmall}/>
            </button>
            <div className={"accordion-body"} style={{display: isCollapsed ? "none" : "block"}}>
                {props.children}
            </div>
        </div>
    )
}