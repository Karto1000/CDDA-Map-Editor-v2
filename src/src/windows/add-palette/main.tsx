import React, {useEffect, useState} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";
import {Tooltip} from "react-tooltip";
import {ImguiSelect, ImguiSelectOption} from "../../shared/components/imguilike/imguiSelect.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {Palette} from "../../tauri/types/palettes.js";
import {BackendResponseType, TauriCommand} from "../../tauri/events/types.js";
import {FormError} from "../../shared/components/form-error.js";

function Main() {
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()
    const [options, setOptions] = useState<ImguiSelectOption[]>([])

    useEffect(() => {
        (async () => {
            const response = await tauriBridge.invoke<{ [id: string]: Palette }, string>(TauriCommand.GET_PALETTES, {})

            if (response.type === BackendResponseType.Error) {
                return
            }

            const newOptions: ImguiSelectOption[] = []
            for (const key of Object.keys(response.data)) {
                const palette  = response.data[key]
                newOptions.push(
                    {
                        value: palette.id,
                        label: palette.id
                    }
                )
            }
            setOptions(newOptions)
        })()
    }, []);

    const onSelectedOptionChange = (selectedOption: string) => {
        console.log(selectedOption)
    }

    function onSubmit(event: React.FormEvent<HTMLFormElement>) {
        event.preventDefault()
    }

    return (
        <GenericWindow title={"Add Palette"}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}/>
            <p>Add a new palette to the current map</p>
            <div className={"line-break"}/>
            <form className={"add-palette-form"} onSubmit={onSubmit}>
                <div className={"form-elements"}>
                    <div className={"form-element"}>
                        <ImguiSelect onChange={onSelectedOptionChange} options={options}/>
                        <label>Palette</label>
                    </div>
                </div>
                <div className={"submit-container"}>
                    <FormError errors={[]}/>
                    <button type={"submit"}>Add Palette</button>
                </div>
            </form>
        </GenericWindow>
    );
}

export default Main;
