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

    initialPosition?: { x: number, y: number }
}

export default function Window(
    {
        title,
        isOpen,
        setIsOpen,
        children,
        initialPosition = {x: 0, y: 0}
    }: Props
) {
    const [isDragging, setIsDragging] = useState(false);
    const [position, setPosition] = useState<{ x: number; y: number }>(initialPosition);
    const dragStartPos = useRef<{ x: number; y: number }>(initialPosition);
    const windowRef = useRef<HTMLDivElement | null>(null);

    const onMouseMove = useCallback((e: MouseEvent) => {
        if (!isDragging) return;

        setPosition({
            x: clamp((position.x + (e.clientX - dragStartPos.current.x)) / window.innerWidth, 0, 1 - windowRef.current.clientWidth / window.innerWidth),
            y: clamp((position.y + (e.clientY - dragStartPos.current.y)) / window.innerHeight, 0, 1 - windowRef.current.clientHeight / window.innerHeight),
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

        dragStartPos.current = {x: normalizedX, y: normalizedY};
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
            style={{left: `${position.x * 100}%`, top: `${position.y * 100}%`}}
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