import {
    InstancedMesh,
    LinearMipMapNearestFilter,
    NearestFilter,
    Object3D,
    SRGBColorSpace,
    Texture,
    TextureLoader,
    Vector2
} from "three";
import {AtlasMaterial} from "./atlas-material.ts";

export type TextureAtlasTile = {
    name: string,
    position: Vector2
}

export type TextureAtlasConfig = {
    tileWidth: number
    tileHeight: number
    atlasWidth: number
    atlasHeight: number,
    maxInstances: number,
    yLayer: number
}

export type InstanceNumber = number;

export class TextureAtlas {
    public readonly textureWidth: number
    public readonly textureHeight: number
    public readonly tileWidth: number
    public readonly tileHeight: number
    public readonly tilesPerRow: number
    public readonly tilesPerColumn: number
    public readonly atlasMaterial: AtlasMaterial
    public readonly maxInstances: number
    public readonly yLayer: number

    public readonly tiles: { [name: string]: TextureAtlasTile }
    public mappedTiles: Map<string, InstanceNumber>
    public mesh: InstancedMesh

    constructor(texture: Texture, tiles: { [name: string]: TextureAtlasTile }, config: TextureAtlasConfig) {
        this.textureWidth = config.atlasWidth
        this.textureHeight = config.atlasHeight
        this.tileWidth = config.tileWidth
        this.tileHeight = config.tileHeight
        this.tilesPerRow = this.textureWidth / this.tileWidth
        this.tilesPerColumn = this.textureHeight / this.tileHeight

        this.atlasMaterial = new AtlasMaterial(this, texture, config.maxInstances)
        this.tiles = tiles
        this.mappedTiles = new Map()
        this.mesh = new InstancedMesh(this.atlasMaterial.geometry, this.atlasMaterial.material, config.maxInstances)
        this.maxInstances = config.maxInstances
        this.yLayer = config.yLayer

        this.mesh.renderOrder = this.yLayer

        for (let instance = 0; instance < config.maxInstances; instance++) {
            const transform = new Object3D()
            // Kinda hacky, but it works for now
            transform.position.set(0, 0, -9999)
            transform.updateMatrix()

            this.mesh.setMatrixAt(instance, transform.matrix)
        }
    }

    public setTileAt(position: Vector2, name: string) {
        let mappedInstance = this.mappedTiles.get(`${position.x}:${position.y}`)

        if (mappedInstance === undefined) {
            mappedInstance = this.atlasMaterial.getNextFreeInstance()
            this.mappedTiles.set(`${position.x}:${position.y}`, mappedInstance)
        }

        const tileToSet = this.tiles[name]
        this.atlasMaterial.setUVAt(mappedInstance, tileToSet.position)

        const transform = new Object3D()
        transform.position.set(position.x * this.tileWidth, position.y * this.tileHeight, this.yLayer)
        transform.updateMatrix()

        this.mesh.instanceMatrix.needsUpdate = true

        this.mesh.setMatrixAt(mappedInstance, transform.matrix)
    }

    public removeTileAt(position: Vector2) {
        let mappedInstance = this.mappedTiles.get(`${position.x}:${position.y}`)

        if (mappedInstance === undefined) return

        const transform = new Object3D()
        // Kinda hacky, but it works for now
        transform.position.set(0, 0, -9999)
        transform.updateMatrix()

        this.mesh.setMatrixAt(mappedInstance, transform.matrix)
        this.mesh.instanceMatrix.needsUpdate = true
    }

    static loadFromURL(url: string, tiles: {
        [name: string]: TextureAtlasTile
    }, config: TextureAtlasConfig): TextureAtlas {
        const texture = new TextureLoader().load(
            url, () => console.log(`Loaded ${url}`),
            () => {
            }, () => {
                throw new Error(`Failed to load ${url}`)
            });

        texture.magFilter = NearestFilter;
        texture.minFilter = LinearMipMapNearestFilter;
        // https://stackoverflow.com/a/77944452
        texture.colorSpace = SRGBColorSpace

        return new TextureAtlas(
            texture,
            tiles,
            config
        )
    }
}