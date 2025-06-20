import React from "react"
import "./contentBox.scss"
import {clsx} from "clsx";

export type ContentBoxProps = {
    children: React.ReactNode[] | React.ReactNode
    className?: string
}

export function ContentBox(props: ContentBoxProps) {
    return (
        <div className={clsx("content-box", props.className)} >
            {props.children}
        </div>
    )
}