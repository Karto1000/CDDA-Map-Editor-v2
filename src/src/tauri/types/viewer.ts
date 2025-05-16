export enum OmTerrainType {
    Single = "Single",
    Nested = "Nested"
}

export type OmTerrain = {
    type: OmTerrainType.Single,
    omTerrainId: string
} | {
    type: OmTerrainType.Nested,
    omTerrainIds: string[][]
}

export type OpenViewerData = {
    filePath: string,
    projectName: string
    omTerrain: OmTerrain
}