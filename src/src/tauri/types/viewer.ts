export enum OpenViewerDataType {
    Terrain = "terrain",
    Special = "special"
}

export type OpenViewerData = {
    type: OpenViewerDataType.Special,
    projectSavePath: string,
    mapgenFilePaths: string[],
    omFilePaths: string[]
    projectName: string
    omId: string
} | {
    type: OpenViewerDataType.Terrain,
    projectSavePath: string,
    mapgenFilePaths: string[],
    projectName: string
    omId: string
}