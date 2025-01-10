import React, {useRef, useEffect, Ref} from "react";

/**
 * @see https://stackoverflow.com/a/42234988
 * Hook that alerts clicks outside of the passed ref
 */
export function useOutsideAlerter(
    target: React.RefObject<any>,
    ignore: React.RefObject<any>[],
    fun: () => void,
) {
    useEffect(() => {
        /**
         * Alert if clicked on outside of element
         */
        function handleClickOutside(event: MouseEvent) {
            let clickedOutside = true;


            if (target.current.contains(event.target)) clickedOutside = false;
            if (ignore.map(i => i.current && i.current.contains(event.target)).some(i => !!i)) clickedOutside = false;

            if (clickedOutside) fun()
        }

        // Bind the event listener
        document.addEventListener("mousedown", handleClickOutside);
        return () => {
            // Unbind the event listener on clean up
            document.removeEventListener("mousedown", handleClickOutside);
        };
    }, [target, ignore]);
}