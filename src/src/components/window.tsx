import React, {Dispatch, useCallback, useEffect, useRef, useState} from "react";
import "./window.scss"
import Icon, {IconName} from "./icon.tsx";

function clamp(value: number, min: number, max: number): number {
    return Math.min(Math.max(value, min), max);
}

type Props = {
    title: string,

    isOpen: boolean;
    setIsOpen: Dispatch<React.SetStateAction<boolean>>;

    children: React.ReactNode[] | React.ReactNode;

    initialPosition?: { x: number, y: number, innerOffsetX: number, innerOffsetY: number }
}

export default function Window(
    {
        title,
        isOpen,
        setIsOpen,
        children,
        initialPosition = {x: 0, y: 0, innerOffsetX: 0, innerOffsetY: 0}
    }: Props
) {
    const [isDragging, setIsDragging] = useState(false);
    const [position, setPosition] = useState<{ x: number; y: number }>(initialPosition);
    const dragStartPos = useRef<{ x: number; y: number, innerOffsetX: number, innerOffsetY: number }>(initialPosition);
    const windowRef = useRef<HTMLDivElement | null>(null);

    const onMouseMove = useCallback((e: MouseEvent) => {
        if (!isDragging) return;

        const normalizedX = (position.x + (e.clientX - dragStartPos.current.x)) / window.innerWidth
        const normalizedY = (position.y + (e.clientY - dragStartPos.current.y)) / window.innerHeight

        const clampMinX = dragStartPos.current.innerOffsetX / window.innerWidth
        const clampMinY = dragStartPos.current.innerOffsetY / window.innerHeight

        const clampMaxX = 1 - (windowRef.current.clientWidth - dragStartPos.current.innerOffsetX) / window.innerWidth
        const clampMaxY = 1 - (windowRef.current.clientHeight - dragStartPos.current.innerOffsetY) / window.innerHeight

        setPosition({
            x: clamp(normalizedX, clampMinX, clampMaxX),
            y: clamp(normalizedY, clampMinY, clampMaxY),
        });
    }, [isDragging, position.x, position.y]);

    const onMouseUp = () => {
        setIsDragging(false);
    };

    const onMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
        e.stopPropagation();
        setIsDragging(true);

        const normalizedX = e.clientX / window.innerWidth;
        const normalizedY = e.clientY / window.innerHeight;
        const innerOffsetX = e.clientX - windowRef.current.offsetLeft
        const innerOffsetY = e.clientY - windowRef.current.offsetTop

        dragStartPos.current = {x: normalizedX, y: normalizedY, innerOffsetX, innerOffsetY};
    };

    useEffect(() => {
        window.addEventListener("mousemove", onMouseMove);
        window.addEventListener("mouseup", onMouseUp);

        return () => {
            window.removeEventListener("mousemove", onMouseMove);
            window.removeEventListener("mouseup", onMouseUp);
        };
    }, [onMouseMove]);

    if (!isOpen) return <></>

    return (
        <div
            className={"window"}
            ref={windowRef}
            style={{
                left: `calc(${position.x * 100}% - ${dragStartPos.current.innerOffsetX}px)`,
                top: `calc(${position.y * 100}% - ${dragStartPos.current.innerOffsetY}px)`
            }}
        >
            <div className={"window-control"} onMouseDown={onMouseDown}>
                <h2>{title}</h2>
                <button className={"close-button"} onClick={() => setIsOpen(false)}>
                    <Icon name={IconName.CloseSmall}/>
                </button>
            </div>
            <div className={"window-content"}>
                {children}
            </div>
        </div>
    )
}