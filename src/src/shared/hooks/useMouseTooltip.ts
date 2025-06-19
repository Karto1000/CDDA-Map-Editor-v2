import React, {useState} from "react";

export type TooltipPosition = { x: number, y: number }
export type MouseMoveHandler = (e: React.MouseEvent<HTMLElement, MouseEvent>) => void

export function useMouseTooltip(): [TooltipPosition, MouseMoveHandler] {
    const [tooltipPosition, setTooltipPosition] = useState<TooltipPosition>({x: 0, y: 0})

    function handleMouseMove(e: React.MouseEvent<HTMLElement, MouseEvent>) {
        setTooltipPosition({x: e.clientX, y: e.clientY})
    }

    return [tooltipPosition, handleMouseMove]
}