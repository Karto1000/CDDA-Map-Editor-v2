import {DrawLocalSprite, Tilesheet} from "./tilesheet.ts";
import {Vector2, Vector3} from "three";
import {TileInfo} from "../../tauri/types/spritesheet.js";
import {RefObject} from "react";
import {ThreeConfig} from "../three/types/three.js";
import {logDeletion} from "../../shared/utils/log.js";

export const MAX_DEPTH = 999997
export const DEFAULT_TILESET = "None"

const MAX_ROW = 1000
const ANIMATION_FRAME_DURATION = 50

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

type StaticBatches = { [zLevel: number]: { [tilesheetName: string]: DrawLocalSprite[] } }
type AnimatedBatches = {
    [zLevel: number]: {
        [tilesheetName: string]: {
            draw: DrawLocalSprite[],
        }
    }
}
type FallbackBatches = { [zLevel: number]: DrawLocalSprite[] }

export class Tilesheets {
    public tilesheets: { [name: string]: Tilesheet }
    public fallback: Tilesheet

    private animatedSprites: SavedAnimatedSprite[] = []

    private cachedStaticBatches: StaticBatches = {}
    private cachedFallbackBatches: FallbackBatches = {}
    private tileInfo: TileInfo

    constructor(tilesheets: { [name: string]: Tilesheet }, fallback: Tilesheet, tileInfo: TileInfo) {
        this.tilesheets = tilesheets
        this.fallback = fallback
        this.tileInfo = tileInfo
    }

    public updateAnimatedSprites(zLevel: number) {
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
                }

                batches[animatedSprite.z][k].draw.push(drawLocalSprite)

                break
            }

            animatedSprite.framesSinceLastDraw = 0
            animatedSprite.currentFrame = nextFrame
        }

        if (!batches[zLevel]) return;

        for (let k of Object.keys(batches[zLevel])) {
            const batch = batches[zLevel][k]

            if (batch.draw.length === 0) continue

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
        for (let k of Object.keys(this.tilesheets)) {
            const tilesheet = this.tilesheets[k]
            tilesheet.clear()
        }

        // Reset the animations here so that the animated sprites update immediately upon switching layers
        for (const animatedSprite of this.animatedSprites) {
            animatedSprite.framesSinceLastDraw = ANIMATION_FRAME_DURATION
            animatedSprite.currentFrame = 0
        }

        if (!this.cachedStaticBatches[zLevel] && !this.cachedFallbackBatches[zLevel]) return

        const staticBatch = this.cachedStaticBatches[zLevel]
        if (staticBatch) {
            for (let k of Object.keys(staticBatch)) {
                const batch = staticBatch[k]
                this.tilesheets[k].drawSpriteLocalIndexBatched(batch)
            }
        }

        const fallbackBatch = this.cachedFallbackBatches[zLevel]
        if (fallbackBatch) this.fallback.drawSpriteLocalIndexBatched(fallbackBatch)

        this.updateAnimatedSprites(zLevel)
    }

    public drawStaticSpritesBatched(staticSprites: DrawStaticSprite[], zLevel: number) {
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

        const currentBatch = batches[zLevel]
        for (let k of Object.keys(currentBatch)) {
            const batch = currentBatch[k]
            this.tilesheets[k].drawSpriteLocalIndexBatched(batch)
        }
    }

    public drawFallbackSpritesBatched(staticSprites: DrawStaticSprite[], zLevel: number) {
        if (staticSprites.length === 0) return

        const batches: FallbackBatches = {}

        for (const drawSprite of staticSprites) {
            const index = drawSprite.index

            const drawLocalSprite = this.getLocalDrawSprite(
                index,
                drawSprite.position,
                drawSprite.layer,
                this.fallback,
                drawSprite.rotate_deg
            )

            if (!batches[drawSprite.z]) batches[drawSprite.z] = []
            batches[drawSprite.z].push(drawLocalSprite)
        }

        this.cachedFallbackBatches = batches
        this.fallback.drawSpriteLocalIndexBatched(batches[zLevel])
    }

    public dispose(threeConfig: RefObject<ThreeConfig>) {
        for (const name of Object.keys(this.tilesheets)) {
            const tilesheet = this.tilesheets[name]
            logDeletion(`[RENDERING] Removing tilesheet ${name} from scene`)
            tilesheet.dispose()
            threeConfig.current.scene.remove(tilesheet.mesh)
        }

        this.fallback.dispose()
        threeConfig.current.scene.remove(this.fallback.mesh)
    }

    public clearAll() {
        for (let k of Object.keys(this.tilesheets)) {
            const tilesheet = this.tilesheets[k]
            tilesheet.clear()
        }

        this.fallback.clear()

        this.cachedStaticBatches = {}
        this.cachedFallbackBatches = {}
        this.animatedSprites = []
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
            index: index - (tilesheet.range ? tilesheet.range[0] : 0),
            layer: layer,
            position: newPosition,
            rotation
        }
    }
}