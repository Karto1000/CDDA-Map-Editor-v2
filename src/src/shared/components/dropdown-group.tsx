import React, {useRef, useState} from "react";
import {Dropdown, DropdownProps, DropdownRef} from "./dropdown.tsx";

type Props = {
    children: React.ReactElement<DropdownProps>[];
}

export function DropdownGroup(props: Props) {
    // null => None are opened
    // number => index of menu that is open
    const [currentlyOpened, setCurrentlyOpened] = useState<number | null>(null);
    // Probably a better way of doing this, but cannot be bothered right now
    // From: https://stackoverflow.com/a/60977523
    // eslint-disable-next-line react-hooks/rules-of-hooks
    const refs = props.children.map(_ => useRef<DropdownRef>(null))

    return (
        <div>
            {
                props.children.map((child, i) => {
                    return <Dropdown
                        name={child.props.name}
                        groups={child.props.groups}
                        ref={refs[i]}
                        onMenuToggle={(opened) => {
                            if (opened) {
                                refs.forEach((ref, refI) => refI !== i && ref.current.closeMenu())
                            }
                            setCurrentlyOpened(opened ? i : null)
                        }}
                        onDropdownButtonHover={() => {
                            if (currentlyOpened === null) return;
                            if (currentlyOpened === i) return;

                            refs[currentlyOpened].current.closeMenu();
                            refs[i].current.openMenu();

                            setCurrentlyOpened(i)
                        }}
                        onOutsideClick={() => {
                            if (currentlyOpened === null) return;

                            if (currentlyOpened === i) {
                                refs[i].current.closeMenu();
                                setCurrentlyOpened(null)
                            }
                        }}
                    />
                })
            }
        </div>
    )
}