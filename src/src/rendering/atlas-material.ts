import {InstancedBufferAttribute, MeshLambertMaterial, PlaneGeometry, Texture, Vector2} from "three";
import {TextureAtlas} from "./texture-atlas.ts";

export class AtlasMaterial {
    public readonly material: MeshLambertMaterial
    public readonly geometry: PlaneGeometry
    public readonly maxInstances: number
    private readonly uvs: Float32Array
    private readonly uvItemSize: number = 2
    private readonly textOffsetAttribute: InstancedBufferAttribute
    private setInstances: Set<number>

    constructor(textureAtlas: TextureAtlas, texture: Texture, maxInstances: number) {
        this.material = new MeshLambertMaterial({map: texture, transparent: true})
        this.geometry = new PlaneGeometry(textureAtlas.tileWidth, textureAtlas.tileHeight)
        this.maxInstances = maxInstances
        this.uvs = new Float32Array(this.maxInstances * this.uvItemSize)

        // Absolute LEGEND -> https://discourse.threejs.org/t/use-texturepacker-atlas-in-an-instancedmesh/63445/6
        this.material.onBeforeCompile = shader => {
            shader.uniforms.atlasSize = {value: new Vector2(textureAtlas.textureWidth, textureAtlas.textureHeight)}
            shader.uniforms.texSize = {value: new Vector2(textureAtlas.tileWidth, textureAtlas.tileHeight)}

            shader.vertexShader = `
                uniform vec2 atlasSize;
                uniform vec2 texSize;
                attribute vec2 texOffset;
                
                ${shader.vertexShader}
            `;

            shader.vertexShader = shader.vertexShader.replace(
                "#include <uv_vertex>",
                `
                #include <uv_vertex>

                // Calculate UV coordinates for the texture atlas
                float uOffset = texOffset.x / atlasSize.x;
                float vOffset = 1.0 - ((texOffset.y + texSize.y) / atlasSize.y);

                vMapUv = (uv * (texSize / atlasSize)) + vec2(uOffset, vOffset);
                `
            );
        }

        this.textOffsetAttribute = new InstancedBufferAttribute(this.uvs, this.uvItemSize, true)
        this.geometry.setAttribute("texOffset", this.textOffsetAttribute);
        this.setInstances = new Set()
    }

    public setUVAt(instance: number, spritesheetTexturePos: Vector2) {
        this.uvs[instance * this.uvItemSize] = spritesheetTexturePos.x
        this.uvs[instance * this.uvItemSize + 1] = spritesheetTexturePos.y
        this.textOffsetAttribute.set(this.uvs)
        this.setInstances.add(instance)

        this.textOffsetAttribute.needsUpdate = true
    }

    public getNextFreeInstance(): number {
        return this.setInstances.size
    }
}