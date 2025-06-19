import React, {useState} from "react";
import GenericWindow from "../generic-window.tsx";
import "./main.scss"
// @ts-ignore
import {SubmitHandler, useForm} from "react-hook-form";
import {FormError} from "../../shared/components/form-error.js";
import {save} from "@tauri-apps/plugin-dialog";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {BackendResponseType, TauriCommand} from "../../tauri/events/types.js";
import {getCurrentWindow} from "@tauri-apps/api/window";
import toast from "react-hot-toast";
import {MultiMenu} from "../../shared/components/imguilike/multimenu.js";
import {Tooltip} from 'react-tooltip'
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";

type SingleMapgenFormInputs = {
    omTerrainName: string
    projectName: string
}


function SingleMapgenForm({handleMouseMove}: {
    handleMouseMove: (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => void
}) {
    const {
        register,
        handleSubmit,
        formState: {errors}
    } = useForm<SingleMapgenFormInputs>();

    const onSubmit: SubmitHandler<SingleMapgenFormInputs> = async (data: SingleMapgenFormInputs) => {
        const entrySavePath = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!entrySavePath) return;

        const projectSavePath = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!projectSavePath) return;

        const response = await tauriBridge.invoke<null, string>(
            TauriCommand.NEW_SINGLE_MAPGEN_VIEWER,
            {
                path: entrySavePath,
                projectSavePath: projectSavePath,
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
        <div className={"new-mapgen-body"}>
            <p>The simplest type of mapgen. A map with a size of 24x24 and no varying z-levels</p>

            <form className={"new-mapgen-form"} onSubmit={handleSubmit(onSubmit)}>
                <div className={"form-elements"}>
                    <div className={"form-element"}>
                        <input
                            type={"text"}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"The overmap terrain name"}
                            onMouseMove={handleMouseMove}
                            placeholder={"Overmap Terrain Name"}
                            {...register("omTerrainName", {required: "Om Terrain name is required"})}
                        />
                        <label>Overmap Name</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            type={"text"}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"The project name; overmap terrain name per default"}
                            onMouseMove={handleMouseMove}
                            placeholder={"Project Name"}
                            {...register("projectName")}
                        />
                        <label>Project Name</label>
                    </div>
                </div>
                <div className={"submit-container"}>
                    <FormError errors={errors}/>
                    <button type={"submit"}>Create</button>
                </div>
            </form>
        </div>

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

function OvermapSpecialForm({handleMouseMove}: {
    handleMouseMove: (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => void
}) {
    const {
        register,
        handleSubmit,
        formState: {errors}
    } = useForm<OvermapSpecialFormInputs>();

    const onSubmit: SubmitHandler<OvermapSpecialFormInputs> = async (data: OvermapSpecialFormInputs) => {
        const entrySavePath = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!entrySavePath) return;

        const projectSavePath = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!projectSavePath) return;

        const response = await tauriBridge.invoke<null, string>(
            TauriCommand.NEW_SPECIAL_MAPGEN_VIEWER,
            {
                path: entrySavePath,
                projectSavePath: projectSavePath,
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
        <div className={"new-mapgen-body"}>
            <p>
                A more complex type of mapgen, consisting of multiple maps which are linked together using an overmap
                special entry. This entry can span multiple z-levels.
            </p>

            <form className={"new-mapgen-form"} onSubmit={handleSubmit(onSubmit)}>
                <div className={"form-elements"}>
                    <div className={"form-element"}>
                        <input
                            type={"text"}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"The overmap terrain name"}
                            onMouseMove={handleMouseMove}
                            placeholder={"Overmap Terrain Name"}
                            {...register("omTerrainName", {required: "Om Terrain name is required"})}
                        />
                        <label>Overmap Name</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            type={"text"}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"The project name; overmap terrain name per default"}
                            onMouseMove={handleMouseMove}
                            placeholder={"Project Name"}
                            {...register("projectName")}
                        />
                        <label>Project Name</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            type={"number"}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"Width of the overmap special"}
                            onMouseMove={handleMouseMove}
                            placeholder={1}
                            {...register("specialWidth", {required: "Special Width is required", valueAsNumber: true})}
                        />
                        <label>Width</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            type={"number"}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"The height of the overmap special"}
                            onMouseMove={handleMouseMove}
                            placeholder={1}
                            {...register("specialHeight", {
                                required: "Special Height is required",
                                valueAsNumber: true
                            })}
                        />
                        <label>Height</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            type={"number"}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"Where the z Level of the overmap special starts"}
                            onMouseMove={handleMouseMove}
                            defaultValue={0}
                            {...register("specialZFrom", {valueAsNumber: true})}
                        />
                        <label>Z-level start</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            type={"number"}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"Where the z Level of the overmap special ends"}
                            onMouseMove={handleMouseMove}
                            defaultValue={0}
                            {...register("specialZTo", {valueAsNumber: true})}
                        />
                        <label>Z-level end</label>
                    </div>
                </div>
                <div className={"submit-container"}>
                    <FormError errors={errors}/>
                    <button type={"submit"}>Create</button>
                </div>
            </form>
        </div>

    )
}

type NestedMapgenFormInputs = {
    omTerrainName: string
    projectName: string
    nestedWidth: number,
    nestedHeight: number
}


function NestedMapgenForm({handleMouseMove}: {
    handleMouseMove: (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => void
}) {
    const {
        register,
        handleSubmit,
        formState: {errors}
    } = useForm<NestedMapgenFormInputs>();

    const onSubmit: SubmitHandler<NestedMapgenFormInputs> = async (data: NestedMapgenFormInputs) => {
        const entrySavePath = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!entrySavePath) return;

        const projectSavePath = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!projectSavePath) return;

        const response = await tauriBridge.invoke<null, string>(
            TauriCommand.NEW_NESTED_MAPGEN_VIEWER,
            {
                path: entrySavePath,
                projectSavePath: projectSavePath,
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
        <div className={"new-mapgen-body"}>
            <p>
                A mapgen entry which is like a single mapgen entry, except for the fact that the size is always between
                1 and 24. These are used to create independent map chunks which can be reused for other maps.
            </p>
            <form className={"new-mapgen-form"} onSubmit={handleSubmit(onSubmit)}>
                <div className={"form-elements"}>
                    <div className={"form-element"}>
                        <input
                            type={"text"}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"The nested overmap terrain id"}
                            onMouseMove={handleMouseMove}
                            placeholder={"Nested mapgen id"}
                            {...register("omTerrainName", {required: "Nested mapgen id is required"})}
                        />
                        <label>Nested Overmap Id</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            type={"text"}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"The project name; nested overmap terrain id per default"}
                            onMouseMove={handleMouseMove}
                            placeholder={"Project Name"}
                            {...register("projectName")}
                        />
                        <label>Project Name</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            type={"number"}
                            min={1}
                            max={24}
                            placeholder={1}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"The width of the nested mapgen; has to be between 1 and 24"}
                            onMouseMove={handleMouseMove}
                            {...register("nestedWidth", {
                                required: "Nested Width is required and must be between 1 and 24",
                                valueAsNumber: true,
                                validate: (v) => v >= 1 && v <= 24
                            })}
                        />
                        <label>Width</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            type={"number"}
                            min={1}
                            max={24}
                            placeholder={1}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-content={"The height of the nested mapgen; has to be between 1 and 24"}
                            onMouseMove={handleMouseMove}
                            {...register("nestedHeight", {
                                required: "Nested Height is required and must be between 1 and 24",
                                valueAsNumber: true,
                                validate: (v) => v >= 1 && v <= 24
                            })}
                        />
                        <label>Height</label>
                    </div>
                </div>
                <div className={"submit-container"}>
                    <FormError errors={errors}/>
                    <button type={"submit"}>Create</button>
                </div>
            </form>
        </div>
    )
}

function NewMapViewer({handleMouseMove}: {
    handleMouseMove: (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => void
}) {
    return (
        <div className={"new-map-viewer-body"}>
            <p>
                Here you can create a new CDDA mapgen file and open it as a Map Viewer.

                <br/>
                <br/>

                Note: If you want to open an already existing Mapgen File, you can use the "Import" functionality.

                <br/>
                <br/>

                When you click on "Create", two file dialogs will open. The first is the location where the new CDDA
                Mapgen entry should be saved.
                The second is where the project file is saved. The project file is the one which is used by the editor
                to remember the location of the the mapgen entry which was just created.
            </p>
            <MultiMenu tabs={
                [
                    {
                        name: "Single Mapgen",
                        content: <SingleMapgenForm handleMouseMove={handleMouseMove}/>
                    },
                    {
                        name: "Nested Mapgen",
                        content: <NestedMapgenForm handleMouseMove={handleMouseMove}/>,
                    },
                    {
                        name: "Overmap Special",
                        content: <OvermapSpecialForm handleMouseMove={handleMouseMove}/>
                    }
                ]
            }/>
        </div>
    )
}

type MapEditorFormInputs = {
    projectName: string
    mapWidth: number
    mapHeight: number
    zLevelFrom: number
    zLevelTo: number
}

function NewMapEditor({handleMouseMove}: {
    handleMouseMove: (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => void
}) {
    const {
        register,
        handleSubmit,
        formState: {errors}
    } = useForm<MapEditorFormInputs>();

    function validateMapSize(v: number): boolean | string {
        if (v < 24) return true
        if (v % 24 !== 0) return "Map sizes greater than 24 must be a multiple of 24"
        return true
    }

    const onSubmit: SubmitHandler<MapEditorFormInputs> = async (data: MapEditorFormInputs) => {
        const path = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!path) return;

        const response = await tauriBridge.invoke<null, string>(
            TauriCommand.NEW_MAP_EDITOR,
            {
                path,
                projectName: data.projectName,
                mapSize: [data.mapWidth, data.mapHeight],
                zLevels: [data.zLevelFrom, data.zLevelTo],
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
        <div className={"new-map-editor-body"}>
            <div className={"new-mapgen-body"}>
                <p>
                    Here you can create a new Map Editor, which in contrast to a Map Viewer, lets you edit maps directly
                    without having to open them in another editor.
                </p>
                <form className={"new-editor-form"} onSubmit={handleSubmit(onSubmit)}>
                    <div className={"form-elements"}>
                        <div className={"form-element"}>
                            <input
                                type={"text"}
                                autoComplete={"off"}
                                data-tooltip-id={"info-tooltip"}
                                data-tooltip-content={"The name of the Project"}
                                onMouseMove={handleMouseMove}
                                placeholder={"Project Name"}
                                {...register("projectName", {required: "Project name is required"})}
                            />
                            <label>Name</label>
                        </div>
                        <div className={"form-element"}>
                            <input
                                type={"number"}
                                placeholder={"24"}
                                defaultValue={24}
                                data-tooltip-id={"info-tooltip"}
                                data-tooltip-content={"The Map width. Any width greater than 24 must be a multiple of 24"}
                                onMouseMove={handleMouseMove}
                                min={1}
                                {...register("mapWidth", {
                                    required: "Map width is required",
                                    valueAsNumber: true,
                                    validate: validateMapSize
                                })}
                            />
                            <label>Width</label>
                        </div>
                        <div className={"form-element"}>
                            <input
                                type={"number"}
                                placeholder={"24"}
                                defaultValue={24}
                                data-tooltip-id={"info-tooltip"}
                                data-tooltip-content={"The Map height. Any height greater than 24 must be a multiple of 24"}
                                onMouseMove={handleMouseMove}
                                min={1}
                                {...register("mapHeight", {
                                    required: "Map height is required",
                                    valueAsNumber: true,
                                    validate: validateMapSize
                                })}
                            />
                            <label>Height</label>
                        </div>
                        <div className={"form-element"}>
                            <input
                                type={"number"}
                                placeholder={"0"}
                                data-tooltip-id={"info-tooltip"}
                                data-tooltip-content={"The lowest z-level"}
                                onMouseMove={handleMouseMove}
                                defaultValue={0}
                                {...register("zLevelFrom", {
                                    required: "ZLevel from is required",
                                    valueAsNumber: true,
                                })}
                            />
                            <label>Lowest Z-level</label>
                        </div>
                        <div className={"form-element"}>
                            <input
                                type={"number"}
                                placeholder={"0"}
                                defaultValue={0}
                                data-tooltip-id={"info-tooltip"}
                                data-tooltip-html="<p>The highest z-level. (Lowest z-level <b>up to and including</b> the highest z-level)</p>"
                                onMouseMove={handleMouseMove}
                                {...register("zLevelTo", {required: "ZLevel To is required", valueAsNumber: true})}
                            />
                            <label>Highest z-level</label>
                        </div>
                    </div>
                    <div className={"submit-container"}>
                        <FormError errors={errors}/>
                        <button type={"submit"}>Create</button>
                    </div>
                </form>
            </div>
        </div>
    )
}

function Main() {
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()

    return (
        <GenericWindow title={"Create new Map"}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}/>
            <MultiMenu tabs={[
                {
                    name: "New Map Editor",
                    content: <NewMapEditor handleMouseMove={handleMouseMove}/>,
                },
                {
                    name: "New Map Viewer",
                    content: <NewMapViewer handleMouseMove={handleMouseMove}/>,
                }
            ]}/>
        </GenericWindow>
    );
}

export default Main;
