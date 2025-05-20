export enum OpenViewerDataType {
    Terrain = "terrain",
    Special = "special"
}

export type OpenViewerData = {
    type: OpenViewerDataType.Special,
    mapgenFilePaths: string[],
    omFilePaths: string[]
    projectName: string
    omId: string
} | {
    type: OpenViewerDataType.Terrain,
    mapgenFilePaths: string[],
    projectName: string
    omId: string
}