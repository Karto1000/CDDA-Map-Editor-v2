import {AtlasMaterial, AtlasMaterialConfig} from "./atlasMaterial.ts";
import {
  InstancedMesh,
  LinearMipMapNearestFilter,
  NearestFilter,
  Object3D, SRGBColorSpace,
  Texture,
  TextureLoader,
  Vector2, Vector3
} from "three";
import {TileInfo, TileNew} from "../lib/tileset/legacy.ts";

export type InstanceNumber = number;

export enum SpriteLayer {
  Fg,
  Bg
}

export type DrawLocalSprite = {
  position: Vector3
  index: number
  layer: number
}

export class Tilesheet {
  public readonly range: [number, number] | null
  public readonly material: AtlasMaterial
  public readonly yLayer: number
  public readonly spritesheetInfo: TileNew
  private atlasConfig: AtlasMaterialConfig

  public mappedTilesFG: Map<string, InstanceNumber>
  public mappedTilesBG: Map<string, InstanceNumber>

  public mesh: InstancedMesh

  constructor(
    texture: Texture,
    tilesetInfo: TileInfo,
    spritesheetInfo: TileNew
  ) {
    const maxInstances = 200000

    const tileWidth = spritesheetInfo.sprite_width || tilesetInfo.width
    const tileHeight = spritesheetInfo.sprite_height || tilesetInfo.height

    let range: [number, number];
    if (spritesheetInfo["//"]) range = Tilesheet.getRangeFromComment(spritesheetInfo, spritesheetInfo["//"])
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
    this.mappedTilesBG = new Map()
    this.mappedTilesFG = new Map()

    for (let instance = 0; instance < this.atlasConfig.maxInstances; instance++) {
      const transform = new Object3D()
      // Kinda hacky, but it works for now
      transform.position.set(0, 0, -999999)
      transform.updateMatrix()

      this.mesh.setMatrixAt(instance, transform.matrix)
    }
  }

  private static getRangeFromComment(spritesheetInfo: TileNew, comment: string): [number, number] | null {
    if (spritesheetInfo["//"]) {
      const split = spritesheetInfo["//"].split(" to ")
      const rangeStart = parseInt(split[0].replace("range ", ""))
      const rangeEnd = parseInt(split[1])
      return [rangeStart, rangeEnd]
    }

    return null;
  }

  private getCoordinatesFromIndex(index: number): Vector2 {
    const localRange = index
    const tilesPerRow = this.atlasConfig.atlasWidth / this.atlasConfig.tileWidth

    const x = localRange % tilesPerRow
    const y = Math.floor(localRange / tilesPerRow)

    return new Vector2(x * this.atlasConfig.tileWidth, y * this.atlasConfig.tileHeight)
  }

  public isWithinRange(index: number): boolean {
    if (!this.range) return false
    return index >= this.range[0] && index <= this.range[1]
  }

  public drawSpriteLocalIndexBatched(drawLocalSprites: DrawLocalSprite[]) {
    const uvMappings = {instances: [], uvs: []}

    for (const drawSprite of drawLocalSprites) {
      const id = `${drawSprite.position.x}:${drawSprite.position.y}:${drawSprite.layer}`

      let mappedInstance = this.mappedTilesBG.get(id)

      if (mappedInstance === undefined) {
        mappedInstance = this.material.getNextFreeInstance()
        this.mappedTilesBG.set(id, mappedInstance)
        this.material.reserveInstance(mappedInstance)
      }

      uvMappings.instances.push(mappedInstance)
      uvMappings.uvs.push(this.getCoordinatesFromIndex(drawSprite.index))

      const transform = new Object3D()

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

  public removeSpriteAtPosition(position: Vector2, layer: SpriteLayer) {
    let mappedInstance = this.mappedTilesBG.get(`${position.x}:${position.y}:${layer}`)

    if (!mappedInstance) return

    this.deleteInstance(mappedInstance)
  }

  public removeSpritesAtPositions(positions: Vector2[], layers: SpriteLayer[]) {
    const mappedInstances = []

    for (let i = 0; i++; i < positions.length) {
      const position = positions[i]
      const layer = layers[i]

      mappedInstances.push(this.mappedTilesBG.get(`${position.x}:${position.y}:${layer}`))
    }

    this.deleteInstances(mappedInstances)
  }

  private deleteInstance(instance: number) {
    this.deleteInstances([instance])
  }


  private deleteInstances(instances: number[]) {
    if (instances.length === 0) return

    const transform = new Object3D()

    // Make it blank
    transform.position.set(
      0,
      0,
      -999999
    )
    transform.updateMatrix()

    for (const instance of instances) {
      this.mappedTilesBG.delete(`${instance}`)
      this.mappedTilesFG.delete(`${instance}`)
      this.mesh.setMatrixAt(instance, transform.matrix)
    }

    this.mesh.instanceMatrix.needsUpdate = true
    this.mesh.computeBoundingSphere()
  }

  public clear() {
    const fgInstances = this.mappedTilesFG
      .keys()
      .map(k => this.mappedTilesFG.get(k))
      .toArray()

    const bgInstances = this.mappedTilesBG
      .keys()
      .map(k => this.mappedTilesBG.get(k))
      .toArray()

    this.deleteInstances([...fgInstances, ...bgInstances])
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
      spritesheetInfo
    )
  }
}