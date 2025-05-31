import React, {RefObject, useImperativeHandle, useRef, useState} from "react";
import "./dropdown.scss"
import {useOutsideAlerter} from "../hooks/useOuterClick.ts";
import Icon, {IconName} from "./icon.tsx";

export type DropdownItem = {
    name: string,
    shortcut?: string,
    expandable?: boolean,
    isToggleable?: boolean,
    toggled?: boolean,
    onToggle?: (state: boolean) => void,
    subGroups?: DropdownItem[][],
    onClick?: (ref: RefObject<DropdownRef>) => void,
}

export type DropdownProps = {
    name: string,
    groups: DropdownItem[][],
    ref?: React.RefObject<DropdownRef>,
    onMenuToggle?: (state: boolean) => void,
    onDropdownButtonHover?: () => void,
    onOutsideClick?: () => void,
}

export type DropdownRef = {
    closeMenu: () => void,
    openMenu: () => void,
}

export function Dropdown(
    {
        name,
        groups,
        onMenuToggle,
        onDropdownButtonHover,
        ref,
        onOutsideClick,
    }: DropdownProps) {
    const [isDropdownOpen, setIsDropdownOpen] = useState(false);
    const [subGroupOpenIndex, setSubGroupOpenIndex] = useState<{ group: number, item: number }>(null);

    const dropdownRef = useRef<HTMLDivElement>(null);
    const menuRef = useRef<HTMLDivElement>(null);

    useOutsideAlerter(
        dropdownRef,
        [menuRef],
        () => {
            setIsDropdownOpen(false);
            setSubGroupOpenIndex(null)

            onOutsideClick()
        }
    )

    useImperativeHandle(
        ref,
        () => {
            return {
                closeMenu: () => setIsDropdownOpen(false),
                openMenu: () => setIsDropdownOpen(true),
            }
        },
        []
    )

    function onDropdownItemMouseEnter(
        groupIndex: number,
        itemIndex: number,
        item: DropdownItem,
        isParentSubgroup: boolean
    ) {
        if (item.subGroups && !isParentSubgroup) setSubGroupOpenIndex(null)
        if (isParentSubgroup) return
        setSubGroupOpenIndex({item: itemIndex, group: groupIndex})
    }

    function getDropdownMenu(sub: boolean, groups: DropdownItem[][]): React.JSX.Element {
        // Holy jank
        return <div className={`dropdown-menu ${sub ? "sub" : ""}`} ref={sub ? null : menuRef}
                    style={{left: sub ? menuRef.current?.clientWidth : 0}} key={`menu${sub ? "-sub" : ""}`}>
            {
                groups.map((items, gi) => (
                    <>
                        <div className={`dropdown-items`}>
                            {
                                items.map((item, ii) => (
                                    <div className={"dropdown-item"}
                                         key={ii}
                                         tabIndex={ii}
                                         onMouseEnter={() => onDropdownItemMouseEnter(gi, ii, item, sub)}
                                         onClick={() => item.onClick(ref)}
                                    >
                                        <div className={"dropdown-item-left"}>
                                            {item.isToggleable ?
                                                <div
                                                    className={`dropdown-item-toggle-button ${item.toggled ? "toggled" : "not-toggled"}`}/>
                                                :
                                                <div className={"fill-space"}/>
                                            }
                                            <span>
                                                {item.name}
                                            </span>
                                        </div>
                                        {
                                            item.expandable &&
                                            <Icon
                                                name={IconName.ChevronUpSmall}
                                                rotation={90}
                                                width={12}
                                                height={12}
                                            />
                                        }
                                        {
                                            item.shortcut &&
                                            <span className={"shortcut"}>{item.shortcut}</span>
                                        }
                                        {subGroupOpenIndex &&
                                            item.subGroups &&
                                            subGroupOpenIndex.group === gi &&
                                            subGroupOpenIndex.item === ii &&
                                            getDropdownMenu(true, item.subGroups)}
                                    </div>
                                ))
                            }
                        </div>
                        {gi < groups.length - 1 &&
                            <div className={"dropdown-section-divider"} key={`div-${gi}`}/>}
                    </>
                ))
            }
        </div>
    }

    return (
        <div className={"dropdown"} ref={dropdownRef}>
            <button className={`dropdown-button ${isDropdownOpen ? "is-dropdown-open" : ""}`}
                    onClick={() => {
                        if (onMenuToggle) onMenuToggle(!isDropdownOpen);
                        setIsDropdownOpen(!isDropdownOpen)
                    }}
                    onMouseOver={() => {
                        if (onDropdownButtonHover) onDropdownButtonHover()
                    }}
            >
                {name}
            </button>

            {
                isDropdownOpen &&
                getDropdownMenu(false, groups)
            }
        </div>
    )
}