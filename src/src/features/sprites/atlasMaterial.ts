import {InstancedBufferAttribute, MeshLambertMaterial, NormalBlending, PlaneGeometry, Texture, Vector2} from "three";

export type AtlasMaterialConfig = {
    tileWidth: number
    tileHeight: number
    atlasWidth: number
    atlasHeight: number,
    maxInstances: number
}

export class AtlasMaterial {
    public readonly material: MeshLambertMaterial
    public readonly geometry: PlaneGeometry
    public readonly maxInstances: number
    private readonly uvs: Float32Array
    private readonly uvItemSize: number = 2
    private readonly textOffsetAttribute: InstancedBufferAttribute
    private setInstances: Set<number>

    constructor(texture: Texture, config: AtlasMaterialConfig) {
        this.material = new MeshLambertMaterial({
            map: texture,
            transparent: true,
            depthWrite: true,
            depthTest: true,
            alphaTest: 0.1,
            blending: NormalBlending
        })
        this.geometry = new PlaneGeometry(config.tileWidth, config.tileHeight)
        this.maxInstances = config.maxInstances
        this.uvs = new Float32Array(this.maxInstances * this.uvItemSize)

        // Absolute LEGEND -> https://discourse.threejs.org/t/use-texturepacker-atlas-in-an-instancedmesh/63445/6
        this.material.onBeforeCompile = shader => {
            shader.uniforms.atlasSize = {value: new Vector2(config.atlasWidth, config.atlasHeight)}
            shader.uniforms.texSize = {value: new Vector2(config.tileWidth, config.tileHeight)}

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
        this.setUVSAt([instance], [spritesheetTexturePos])
    }

    public reserveInstance(instance: number) {
        this.setInstances.add(instance)
    }

    public setUVSAt(instances: number[], spritesheetTexturesPos: Vector2[]) {
        for (let i = 0; i < instances.length; i++) {
            const instance = instances[i]
            const texturePos = spritesheetTexturesPos[i]

            const uv = new Vector2(texturePos.x, texturePos.y)
            this.uvs[instance * this.uvItemSize] = uv.x
            this.uvs[instance * this.uvItemSize + 1] = uv.y
        }

        this.textOffsetAttribute.set(this.uvs)
        this.textOffsetAttribute.needsUpdate = true
    }

    public getNextFreeInstance(): number {
        return this.setInstances.size
    }
}