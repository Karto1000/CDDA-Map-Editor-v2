import React, {CSSProperties, useState} from "react";
import "./fieldset.scss"

export type FieldsetProps = {
    legend: string,
    children: React.ReactNode[] | React.ReactNode,

    className?: string
    style?: CSSProperties
}

export function Fieldset(props: FieldsetProps) {
    const [isCollapsed, setIsCollapsed] = useState<boolean>(false)

    return (
        <fieldset className={`fieldset ${props.className ? props.className : ''} ${isCollapsed ? "collapsed" : ""}`} style={props.style}>
            <legend className={"collapser"} onClick={() => setIsCollapsed(!isCollapsed)}>{props.legend}</legend>
            <div className={`collapsable`}>
                {props.children}
            </div>
        </fieldset>
    )
}