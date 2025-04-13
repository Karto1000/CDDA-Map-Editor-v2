import {DrawLocalSprite, SpriteLayer, Tilesheet} from "./tilesheet.ts";
import {Vector2, Vector3} from "three";

const MAX_DEPTH = 999997
const TILE_SIZE = 32
const MAX_ROW = 1000
const ANIMATION_FRAME_DURATION = 200

export type DrawStaticSprite = {
  position: Vector2
  index: number
  layer: number
}

export type DrawAnimatedSprite = {
  position: Vector2
  indices: number[],
  layer: number
}

type SavedAnimatedSprite = DrawAnimatedSprite & {
  framesSinceLastDraw: number
  currentFrame: number
}

export class Tilesheets {
  public tilesheets: { [name: string]: Tilesheet }
  public fallback: Tilesheet
  private animatedSprites: SavedAnimatedSprite[]

  constructor(tilesheets: { [name: string]: Tilesheet }, fallback: Tilesheet) {
    this.tilesheets = tilesheets
    this.fallback = fallback
  }

  public updateAnimatedSprites() {
    const batches: {
      [key: string]: { draw: DrawLocalSprite[], remove: { positions: Vector2[], layers: SpriteLayer[] } }
    } = {}

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
          tilesheet
        )

        if (!batches[k]) batches[k] = {draw: [], remove: {positions: [], layers: []}}

        batches[k].draw.push(drawLocalSprite)
        batches[k].remove.positions.push(animatedSprite.position)
        batches[k].remove.layers.push(animatedSprite.layer)

        break
      }

      animatedSprite.framesSinceLastDraw = 0
      animatedSprite.currentFrame = nextFrame
    }

    for (let k of Object.keys(batches)) {
      const batch = batches[k]

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

  private getLocalDrawSprite(index: number, position: Vector2, layer: number, tilesheet: Tilesheet): DrawLocalSprite {
    const worldY = position.y / TILE_SIZE
    const worldX = position.x / TILE_SIZE

    const newPosition = new Vector3(
      position.x,
      position.y,
      // + 1 to always add an offset because if we didn't, a few sprites would not show up
      (MAX_DEPTH - MAX_ROW * (worldY + 1)) + worldX + layer
    )

    return {
      index: index - tilesheet.range[0],
      layer: layer,
      position: newPosition
    }
  }

  public drawStaticSpritesBatched(staticSprites: DrawStaticSprite[]) {
    const batches: { [key: string]: DrawLocalSprite[] } = {}

    for (const drawSprite of staticSprites) {
      const index = drawSprite.index

      for (let k of Object.keys(this.tilesheets)) {
        const tilesheet = this.tilesheets[k]
        if (!tilesheet.isWithinRange(index)) continue

        const drawLocalSprite = this.getLocalDrawSprite(
          index,
          drawSprite.position,
          drawSprite.layer,
          tilesheet
        )

        if (!batches[k]) batches[k] = []
        batches[k].push(drawLocalSprite)

        break
      }
    }

    for (let k of Object.keys(batches)) {
      const batch = batches[k]
      this.tilesheets[k].drawSpriteLocalIndexBatched(batch)
    }
  }

  public drawFallbackSpritesBatched(staticSprites: DrawStaticSprite[]) {
    const batch = []

    for (const drawSprite of staticSprites) {
      const index = drawSprite.index

      const worldY = drawSprite.position.y / TILE_SIZE
      const worldX = drawSprite.position.x / TILE_SIZE

      const newPosition = new Vector3(
        drawSprite.position.x,
        drawSprite.position.y,
        // + 1 to always add an offset because if we didn't, a few sprites would not show up
        (MAX_DEPTH - MAX_ROW * (worldY + 1)) + worldX + drawSprite.layer
      )

      const drawLocalSprite: DrawLocalSprite = {
        index: index,
        layer: drawSprite.layer,
        position: newPosition
      }

      batch.push(drawLocalSprite)
    }

    this.fallback.drawSpriteLocalIndexBatched(batch)
  }

  public clearAll() {
    for (let k of Object.keys(this.tilesheets)) {
      const tilesheet = this.tilesheets[k]
      tilesheet.clear()
    }

    this.animatedSprites = []
  }

  public removeSprite(position: Vector2, layer: SpriteLayer) {
    for (let k of Object.keys(this.tilesheets)) {
      const tilesheet = this.tilesheets[k]
      tilesheet.removeSpriteAtPosition(position, layer)
    }
  }
}