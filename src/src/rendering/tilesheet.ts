import {AtlasMaterial, AtlasMaterialConfig} from "./atlasMaterial.ts";
import {
    InstancedMesh,
    LinearMipMapNearestFilter,
    NearestFilter,
    Object3D, SRGBColorSpace,
    Texture,
    TextureLoader,
    Vector2
} from "three";

export type InstanceNumber = number;

export class Tilesheet {
    public readonly range: [number, number]
    public readonly material: AtlasMaterial
    public readonly yLayer: number
    private atlasConfig: AtlasMaterialConfig
    public mappedTiles: Map<string, InstanceNumber>
    public mesh: InstancedMesh

    constructor(
        texture: Texture,
        tileWidth: number,
        tileHeight: number,
        range: [number, number]
    ) {
        const maxInstances = 1

        const atlasMaterialConfig = {
            tileWidth: tileWidth,
            tileHeight: tileHeight,
            atlasWidth: texture.image.width,
            atlasHeight: texture.image.height,
            maxInstances
        }

        this.range = range
        this.yLayer = 0
        this.material = new AtlasMaterial(
            texture,
            atlasMaterialConfig
        )
        this.atlasConfig = atlasMaterialConfig
        this.mesh = new InstancedMesh(
            this.material.geometry,
            this.material.material,
            maxInstances
        )
        this.mesh.renderOrder = this.yLayer
        this.mappedTiles = new Map()

        for (let instance = 0; instance < this.atlasConfig.maxInstances; instance++) {
            const transform = new Object3D()
            // Kinda hacky, but it works for now
            transform.position.set(0, 0, -9999)
            transform.updateMatrix()

            this.mesh.setMatrixAt(instance, transform.matrix)
        }
    }

    private getCoordinatesFromIndex(index: number): Vector2 {
        const localRange = index
        const tilesPerRow = this.atlasConfig.atlasWidth / this.atlasConfig.tileWidth

        const x = localRange % tilesPerRow
        const y = Math.floor(localRange / tilesPerRow)

        return new Vector2(x * this.atlasConfig.tileWidth, y * this.atlasConfig.tileHeight)
    }

    public isWithinRange(index: number): boolean {
        return index >= this.range[0] && index <= this.range[1]
    }

    public drawSpriteLocalIndex(index: number, position: Vector2) {
        let mappedInstance = this.mappedTiles.get(`${position.x}:${position.y}`)

        if (mappedInstance === undefined) {
            mappedInstance = this.material.getNextFreeInstance()
            this.mappedTiles.set(`${position.x}:${position.y}`, mappedInstance)
        }

        this.material.setUVAt(mappedInstance, this.getCoordinatesFromIndex(index))

        const transform = new Object3D()
        transform.position.set(
            position.x * this.atlasConfig.tileWidth,
            position.y * this.atlasConfig.tileHeight,
            this.yLayer
        )
        transform.updateMatrix()

        this.mesh.setMatrixAt(mappedInstance, transform.matrix)
        this.mesh.instanceMatrix.needsUpdate = true
        this.mesh.computeBoundingSphere()
    }

    public static async fromURL(
        url: string,
        tilesetInfo: TileInfo,
        spritesheetInfo: TileNew
    ): Promise<Tilesheet> {
        const texture = await new TextureLoader()
            .loadAsync(url, () => console.log(`Loading ${url}`))

        texture.magFilter = NearestFilter;
        texture.minFilter = LinearMipMapNearestFilter;
        // https://stackoverflow.com/a/77944452
        texture.colorSpace = SRGBColorSpace

        const tileWidth = spritesheetInfo.sprite_width || tilesetInfo.width
        const tileHeight = spritesheetInfo.sprite_height || tilesetInfo.height

        let range: [number, number];
        if (spritesheetInfo["//"]) {
            const split = spritesheetInfo["//"].split(" to ")
            const rangeStart = parseInt(split[0].replace("range ", ""))
            const rangeEnd = parseInt(split[1])
            range = [rangeStart, rangeEnd]
        } else {
            range = [0, 0]
        }

        return new Tilesheet(
            texture,
            tileWidth,
            tileHeight,
            range
        )
    }
}