import {DrawLocalSprite, Tilesheet} from "./tilesheet.ts";
import {Vector2, Vector3} from "three";
import {TileInfo} from "../lib/tileset/legacy.js";

export const MAX_DEPTH = 999997
const MAX_ROW = 1000
const ANIMATION_FRAME_DURATION = 200

export type DrawStaticSprite = {
    position: Vector2
    index: number
    layer: number
    rotate_deg: number
    z: number
}

export type DrawAnimatedSprite = {
    position: Vector2
    indices: number[],
    layer: number
    rotate_deg: number
    z: number
}

type SavedAnimatedSprite = DrawAnimatedSprite & {
    framesSinceLastDraw: number
    currentFrame: number
}

type StaticBatches = { [zLevel: number]: { [key: string]: DrawLocalSprite[] } }
type AnimatedBatches = {
    [zLevel: number]: {
        [key: string]: {
            draw: DrawLocalSprite[],
            remove: { positions: Vector2[], layers: number[] }
        }
    }
}
type FallbackBatches = { [zLevel: number]: DrawLocalSprite[] }

export class Tilesheets {
    public tilesheets: { [name: string]: Tilesheet }
    public fallback: Tilesheet

    private zLevel: number = 0
    private animatedSprites: SavedAnimatedSprite[]

    private cachedStaticBatches: StaticBatches = {}
    private cachedFallbackBatches: FallbackBatches = {}
    private tileInfo: TileInfo

    constructor(tilesheets: { [name: string]: Tilesheet }, fallback: Tilesheet, tileInfo: TileInfo) {
        this.tilesheets = tilesheets
        this.fallback = fallback
        this.tileInfo = tileInfo
    }

    public updateAnimatedSprites() {
        if (this.animatedSprites.length === 0) return

        const batches: AnimatedBatches = {}

        for (const animatedSprite of this.animatedSprites) {
            if (animatedSprite.framesSinceLastDraw <= ANIMATION_FRAME_DURATION) {
                animatedSprite.framesSinceLastDraw += 1
                continue
            }

            let nextFrame = animatedSprite.currentFrame + 1;
            if (nextFrame >= animatedSprite.indices.length) nextFrame = 0

            for (let k of Object.keys(this.tilesheets)) {
                const tilesheet = this.tilesheets[k]
                if (!tilesheet.isWithinRange(animatedSprite.indices[nextFrame])) continue

                const drawLocalSprite = this.getLocalDrawSprite(
                    animatedSprite.indices[animatedSprite.currentFrame],
                    animatedSprite.position,
                    animatedSprite.layer,
                    tilesheet,
                    animatedSprite.rotate_deg
                )

                if (!batches[animatedSprite.z]) batches[animatedSprite.z] = {}
                if (!batches[animatedSprite.z][k]) batches[animatedSprite.z][k] = {
                    draw: [],
                    remove: {positions: [], layers: []}
                }

                batches[animatedSprite.z][k].draw.push(drawLocalSprite)
                batches[animatedSprite.z][k].remove.positions.push(animatedSprite.position)
                batches[animatedSprite.z][k].remove.layers.push(animatedSprite.layer)

                break
            }

            animatedSprite.framesSinceLastDraw = 0
            animatedSprite.currentFrame = nextFrame
        }

        if (!batches[this.zLevel]) return;

        for (let k of Object.keys(batches[this.zLevel])) {
            const batch = batches[this.zLevel][k]

            if (batch.draw.length === 0) continue
            if (batch.remove.positions.length === 0) continue

            this.tilesheets[k].drawSpriteLocalIndexBatched(batch.draw)
        }
    }

    public drawAnimatedSpritesBatched(animatedSprites: DrawAnimatedSprite[]) {
        this.animatedSprites.push(...animatedSprites.map(s => {
                return {...s, framesSinceLastDraw: ANIMATION_FRAME_DURATION, currentFrame: 0}
            })
        )
    }

    public switchZLevel(zLevel: number) {
        this.clearAll()

        this.zLevel = zLevel

        if (!this.cachedStaticBatches[this.zLevel] && !this.cachedFallbackBatches[this.zLevel]) return

        const staticBatch = this.cachedStaticBatches[this.zLevel]
        if (staticBatch) {
            for (let k of Object.keys(staticBatch)) {
                const batch = staticBatch[k]
                this.tilesheets[k].drawSpriteLocalIndexBatched(batch)
            }
        }

        const fallbackBatch = this.cachedFallbackBatches[this.zLevel]
        if (fallbackBatch) this.fallback.drawSpriteLocalIndexBatched(fallbackBatch)
    }

    public drawStaticSpritesBatched(staticSprites: DrawStaticSprite[]) {
        if (staticSprites.length === 0) return

        const batches: StaticBatches = {}

        for (const staticSprite of staticSprites) {
            const index = staticSprite.index

            for (let k of Object.keys(this.tilesheets)) {
                const tilesheet = this.tilesheets[k]
                if (!tilesheet.isWithinRange(index)) continue

                const drawLocalSprite = this.getLocalDrawSprite(
                    index,
                    staticSprite.position,
                    staticSprite.layer,
                    tilesheet,
                    staticSprite.rotate_deg
                )

                if (!batches[staticSprite.z]) batches[staticSprite.z] = {}
                if (!batches[staticSprite.z][k]) batches[staticSprite.z][k] = []
                batches[staticSprite.z][k].push(drawLocalSprite)

                break
            }
        }

        this.cachedStaticBatches = batches

        const currentBatch = batches[this.zLevel]
        for (let k of Object.keys(currentBatch)) {
            const batch = currentBatch[k]
            this.tilesheets[k].drawSpriteLocalIndexBatched(batch)
        }
    }

    public drawFallbackSpritesBatched(staticSprites: DrawStaticSprite[]) {
        if (staticSprites.length === 0) return

        const batches: FallbackBatches = {}

        for (const drawSprite of staticSprites) {
            const index = drawSprite.index

            const worldY = drawSprite.position.y / this.tileInfo.width
            const worldX = drawSprite.position.x / this.tileInfo.height

            const newPosition = new Vector3(
                drawSprite.position.x,
                drawSprite.position.y,
                // + 1 to always add an offset because if we didn't, a few sprites would not show up
                (MAX_DEPTH - MAX_ROW * (worldY + 1)) + worldX + drawSprite.layer
            )

            const drawLocalSprite: DrawLocalSprite = {
                index: index,
                layer: drawSprite.layer,
                position: newPosition,
                rotation: drawSprite.rotate_deg
            }

            if (!batches[drawSprite.z]) batches[drawSprite.z] = []
            batches[drawSprite.z].push(drawLocalSprite)
        }

        this.cachedFallbackBatches = batches
        this.fallback.drawSpriteLocalIndexBatched(batches[this.zLevel])
    }

    public clearAll() {
        for (let k of Object.keys(this.tilesheets)) {
            const tilesheet = this.tilesheets[k]
            tilesheet.clear()
        }

        this.animatedSprites = []
    }

    public removeSprite(position: Vector2, layer: number) {
        for (let k of Object.keys(this.tilesheets)) {
            const tilesheet = this.tilesheets[k]
            tilesheet.removeSpriteAtPosition(position, layer)
        }
    }

    private getLocalDrawSprite(
        index: number,
        position: Vector2,
        layer: number,
        tilesheet: Tilesheet,
        rotation: number
    ): DrawLocalSprite {
        const worldY = position.y / this.tileInfo.width
        const worldX = position.x / this.tileInfo.height

        // Since the three.js world goes from down to up and our cdda map goes from up to down, we need to invert the
        // cell y position
        const newPosition = new Vector3(
            position.x,
            -position.y - this.tileInfo.height,
            // + 1 to always add an offset because if we didn't, a few sprites would not show up
            (MAX_ROW * (worldY + 1)) + worldX + layer
        )

        return {
            index: index - tilesheet.range[0],
            layer: layer,
            position: newPosition,
            rotation
        }
    }
}