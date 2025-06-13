import {AtlasMaterial, AtlasMaterialConfig} from "./atlasMaterial.ts";
import {
    InstancedMesh,
    LinearMipMapNearestFilter, Material,
    NearestFilter,
    Object3D,
    SRGBColorSpace,
    Texture,
    TextureLoader,
    Vector2,
    Vector3
} from "three";
import {TileInfo, TileNew} from "../../tauri/types/spritesheet.js";

export type InstanceNumber = number;

function degreesToRadians(degrees: number) {
    const pi = Math.PI;
    return degrees * (pi / 180);
}

export type DrawLocalSprite = {
    position: Vector3
    index: number
    rotation: number
    layer: number
}

export class Tilesheet {
    public readonly range: [number, number] | null
    public readonly material: AtlasMaterial
    public readonly yLayer: number
    public readonly spritesheetInfo: TileNew
    public readonly objectURL: string
    public mappedTiles: Map<string, InstanceNumber>
    public mesh: InstancedMesh
    private atlasConfig: AtlasMaterialConfig

    constructor(
        texture: Texture,
        tilesetInfo: TileInfo,
        spritesheetInfo: TileNew,
        objectUrl: string
    ) {
        this.objectURL = objectUrl

        const maxInstances = 200_000

        const tileWidth = spritesheetInfo.sprite_width || tilesetInfo.width
        const tileHeight = spritesheetInfo.sprite_height || tilesetInfo.height

        let range: [number, number];
        if (spritesheetInfo["//"]) range = spritesheetInfo["//"]
        else range = null

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
        this.spritesheetInfo = spritesheetInfo
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
            transform.position.set(0, 0, -999999)
            transform.updateMatrix()

            this.mesh.setMatrixAt(instance, transform.matrix)
        }
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

        return new Tilesheet(
            texture,
            tilesetInfo,
            spritesheetInfo,
            url
        )
    }

    public isWithinRange(index: number): boolean {
        if (!this.range) return false
        return index >= this.range[0] && index <= this.range[1]
    }

    public drawSpriteLocalIndexBatched(drawLocalSprites: DrawLocalSprite[]) {
        if (drawLocalSprites.length === 0) return

        const uvMappings = {instances: [], uvs: []}

        for (const drawSprite of drawLocalSprites) {
            const id = `${drawSprite.position.x}:${drawSprite.position.y}:${drawSprite.layer}`

            let mappedInstance = this.mappedTiles.get(id)

            if (mappedInstance === undefined) {
                mappedInstance = this.material.getNextFreeInstance()
                this.mappedTiles.set(id, mappedInstance)
                this.material.reserveInstance(mappedInstance)
            }

            uvMappings.instances.push(mappedInstance)
            uvMappings.uvs.push(this.getCoordinatesFromIndex(drawSprite.index))

            const transform = new Object3D()
            transform.rotateZ(degreesToRadians(drawSprite.rotation))

            transform.position.set(
                drawSprite.position.x,
                drawSprite.position.y - (this.spritesheetInfo.sprite_offset_y || 0) / 2,
                drawSprite.position.z
            )
            transform.updateMatrix()

            this.mesh.setMatrixAt(mappedInstance, transform.matrix)
        }

        this.material.setUVSAt(uvMappings.instances, uvMappings.uvs)
        this.mesh.instanceMatrix.needsUpdate = true
        this.mesh.computeBoundingSphere()
    }

    public clear() {
        const tiles = this.mappedTiles
            .keys()
            .map(k => this.mappedTiles.get(k))
            .toArray()

        this.deleteInstances([...tiles])
    }

    public dispose() {
        this.mesh.dispose()
        this.material.dispose()
    }

    private getCoordinatesFromIndex(index: number): Vector2 {
        const localRange = index
        const tilesPerRow = this.atlasConfig.atlasWidth / this.atlasConfig.tileWidth

        const x = localRange % tilesPerRow
        const y = Math.floor(localRange / tilesPerRow)

        return new Vector2(x * this.atlasConfig.tileWidth, y * this.atlasConfig.tileHeight)
    }

    private deleteInstances(instances: number[]) {
        if (instances.length === 0) return

        this.mesh.clear()

        const transform = new Object3D()

        // Make it blank
        transform.position.set(
            0,
            0,
            -999999
        )
        transform.updateMatrix()

        for (const instance of instances) {
            this.mappedTiles.delete(`${instance}`)
            this.mesh.setMatrixAt(instance, transform.matrix)
        }

        this.mesh.instanceMatrix.needsUpdate = true
        this.mesh.computeBoundingSphere()
    }
}