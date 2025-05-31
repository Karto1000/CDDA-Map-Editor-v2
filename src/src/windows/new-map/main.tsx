import React from "react";
import GenericWindow from "../generic-window.tsx";
import "./main.scss"
import {MultiMenu} from "../../shared/components/multimenu.tsx";
import {SubmitHandler, useForm} from "react-hook-form";
import {FormError} from "../../shared/components/form-error.js";
import {save} from "@tauri-apps/plugin-dialog";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {TauriCommand} from "../../tauri/events/types.js";
import {getCurrentWindow} from "@tauri-apps/api/window";

type SingleMapgenFormInputs = {
    omTerrainName: string
    projectName: string
}


function SingleMapgenForm() {
    const {
        register,
        handleSubmit,
        formState: {errors}
    } = useForm<SingleMapgenFormInputs>();

    const onSubmit: SubmitHandler<SingleMapgenFormInputs> = async (data: SingleMapgenFormInputs) => {
        const path = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!path) return;

        await tauriBridge.invoke<null, string, TauriCommand.NEW_SINGLE_MAPGEN_VIEWER>(
            TauriCommand.NEW_SINGLE_MAPGEN_VIEWER,
            {
                path: path,
                omTerrainName: data.omTerrainName,
                projectName: data.projectName ? data.projectName : data.omTerrainName,
            }
        )

        const window = getCurrentWindow();
        await window.close()
    }

    return (
        <form className={"new-mapgen-form"} onSubmit={handleSubmit(onSubmit)}>
            <div className={"form-elements"}>
                <div className={"form-element"}>
                    <input
                        type={"text"}
                        placeholder={"Overmap Terrain Name"}
                        {...register("omTerrainName", {required: "Om Terrain name is required"})}
                    />
                    <label>Overmap Terrain Name</label>
                </div>
                <div className={"form-element"}>
                    <input
                        type={"text"}
                        placeholder={"Project Name"}
                        {...register("projectName")}
                    />
                    <label>Project Name, default is Om Terrain name</label>
                </div>
            </div>
            <div className={"submit-container"}>
                <FormError errors={errors}/>
                <button type={"submit"}>Create</button>
            </div>
        </form>
    )
}

type OvermapSpecialFormInputs = {
    omTerrainName: string
    projectName: string
    specialWidth: number,
    specialHeight: number
    specialZFrom: number,
    specialZTo: number,
}

function OvermapSpecialForm() {
    const {
        register,
        handleSubmit,
        formState: {errors}
    } = useForm<OvermapSpecialFormInputs>();

    const onSubmit: SubmitHandler<OvermapSpecialFormInputs> = async (data: OvermapSpecialFormInputs) => {
        const path = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!path) return;

        console.log(data)
        await tauriBridge.invoke<null, string, TauriCommand.NEW_SPECIAL_MAPGEN_VIEWER>(
            TauriCommand.NEW_SPECIAL_MAPGEN_VIEWER,
            {
                path: path,
                omTerrainName: data.omTerrainName,
                projectName: data.projectName ? data.projectName : data.omTerrainName,
                specialWidth: data.specialWidth,
                specialHeight: data.specialHeight,
                specialZFrom: data.specialZFrom,
                specialZTo: data.specialZTo
            }
        )

        const window = getCurrentWindow();
        await window.close()
    }

    return (
        <form className={"new-mapgen-form"} onSubmit={handleSubmit(onSubmit)}>
            <div className={"form-elements"}>
                <div className={"form-element"}>
                    <input
                        type={"text"}
                        placeholder={"Overmap Terrain Name"}
                        {...register("omTerrainName", {required: "Om Terrain name is required"})}
                    />
                    <label>Overmap Terrain Name</label>
                </div>
                <div className={"form-element"}>
                    <input
                        type={"text"}
                        placeholder={"Project Name"}
                        {...register("projectName")}
                    />
                    <label>Project Name, default is Om Terrain name</label>
                </div>
                <div className={"form-element"}>
                    <input
                        type={"number"}
                        {...register("specialWidth", {required: "Special Width is required", valueAsNumber: true})}
                    />
                    <label>Width of the overmap special</label>
                </div>
                <div className={"form-element"}>
                    <input
                        type={"number"}
                        {...register("specialHeight", {required: "Special Height is required", valueAsNumber: true})}
                    />
                    <label>Height of the overmap special</label>
                </div>
                <div className={"form-element"}>
                    <input
                        type={"number"}
                        defaultValue={0}
                        {...register("specialZFrom", {valueAsNumber: true})}
                    />
                    <label>Where the z Level of the overmap special starts</label>
                </div>
                <div className={"form-element"}>
                    <input
                        type={"number"}
                        defaultValue={0}
                        {...register("specialZTo", {valueAsNumber: true})}
                    />
                    <label>Where the z Level of the overmap special ends</label>
                </div>
            </div>
            <div className={"submit-container"}>
                <FormError errors={errors}/>
                <button type={"submit"}>Create</button>
            </div>
        </form>
    )
}

function NewMapViewer() {
    return (
        <div className={"new-map-viewer-body"}>
            <p>
                Here you can create a new CDDA mapgen file and open it as a Map Viewer.
            </p>
            <MultiMenu tabs={
                [
                    {
                        name: "Single Mapgen",
                        content: <SingleMapgenForm/>,
                    },
                    {
                        name: "Nested Mapgen",
                        content: <></>,
                        isDisabled: true
                    },
                    {
                        name: "Overmap Special",
                        content: <OvermapSpecialForm/>
                    }
                ]
            }/>
        </div>
    )
}

function Main() {
    return (
        <GenericWindow title={"Create new Map"}>
            <MultiMenu tabs={[
                {
                    name: "New Map Editor",
                    content: <></>,
                    isDisabled: true
                },
                {
                    name: "New Map Viewer",
                    content: <NewMapViewer/>,
                }
            ]}/>
        </GenericWindow>
    );
}

export default Main;
