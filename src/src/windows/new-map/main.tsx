import React from "react";
import GenericWindow from "../generic-window.tsx";
import "./main.scss"
import {MultiMenu} from "../../shared/components/multimenu.tsx";
import {SubmitHandler, useForm} from "react-hook-form";
import {FormError} from "../../shared/components/form-error.js";
import {save} from "@tauri-apps/plugin-dialog";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {BackendResponseType, TauriCommand} from "../../tauri/events/types.js";
import {getCurrentWindow} from "@tauri-apps/api/window";
import toast from "react-hot-toast";

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

        const response = await tauriBridge.invoke<null, string, TauriCommand.NEW_SINGLE_MAPGEN_VIEWER>(
            TauriCommand.NEW_SINGLE_MAPGEN_VIEWER,
            {
                path: path,
                omTerrainName: data.omTerrainName,
                projectName: data.projectName ? data.projectName : data.omTerrainName,
            }
        )

        if (response.type === BackendResponseType.Error) {
            toast.error(response.error)
            return
        }

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

        const response = await tauriBridge.invoke<null, string, TauriCommand.NEW_SPECIAL_MAPGEN_VIEWER>(
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

        if (response.type === BackendResponseType.Error) {
            toast.error(response.error)
            return
        }

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
                        placeholder={1}
                        {...register("specialWidth", {required: "Special Width is required", valueAsNumber: true})}
                    />
                    <label>Width of the overmap special</label>
                </div>
                <div className={"form-element"}>
                    <input
                        type={"number"}
                        placeholder={1}
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

type NestedMapgenFormInputs = {
    omTerrainName: string
    projectName: string
    nestedWidth: number,
    nestedHeight: number
}


function NestedMapgenForm() {
    const {
        register,
        handleSubmit,
        formState: {errors}
    } = useForm<NestedMapgenFormInputs>();

    const onSubmit: SubmitHandler<NestedMapgenFormInputs> = async (data: NestedMapgenFormInputs) => {
        const path = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!path) return;

        const response = await tauriBridge.invoke<null, string, TauriCommand.NEW_NESTED_MAPGEN_VIEWER>(
            TauriCommand.NEW_NESTED_MAPGEN_VIEWER,
            {
                path: path,
                omTerrainName: data.omTerrainName,
                projectName: data.projectName ? data.projectName : data.omTerrainName,
                nestedWidth: data.nestedWidth,
                nestedHeight: data.nestedHeight,
            }
        )

        if (response.type === BackendResponseType.Error) {
            toast.error(response.error)
            return
        }

        const window = getCurrentWindow();
        await window.close()
    }

    return (
        <form className={"new-mapgen-form"} onSubmit={handleSubmit(onSubmit)}>
            <div className={"form-elements"}>
                <div className={"form-element"}>
                    <input
                        type={"text"}
                        placeholder={"Nested mapgen id"}
                        {...register("omTerrainName", {required: "Nested mapgen id is required"})}
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
                        min={1}
                        max={24}
                        placeholder={1}
                        {...register("nestedWidth", {
                            required: "Nested Width is required and must be between 1 and 24",
                            valueAsNumber: true,
                            validate: (v) => v >= 1 && v <= 24
                        })}
                    />
                    <label>Width of the nested mapgen, has to be between 1 and 24</label>
                </div>
                <div className={"form-element"}>
                    <input
                        type={"number"}
                        min={1}
                        max={24}
                        placeholder={1}
                        {...register("nestedHeight", {
                            required: "Nested Height is required and must be between 1 and 24",
                            valueAsNumber: true,
                            validate: (v) => v >= 1 && v <= 24
                        })}
                    />
                    <label>Height of the nested mapgen, has to be between 1 and 24</label>
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
                        content: <NestedMapgenForm/>,
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
