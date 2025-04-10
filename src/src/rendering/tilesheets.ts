import {SpriteLayer, Tilesheet} from "./tilesheet.ts";
import {Vector2, Vector3} from "three";

const MAX_DEPTH = 999997
const TILE_SIZE = 32
const MAX_ROW = 300

export class Tilesheets {
    public tilesheets: { [name: string]: Tilesheet }

    constructor(tilesheets: { [name: string]: Tilesheet }) {
        this.tilesheets = tilesheets
    }

    public drawSprite(index: number, position: Vector2, layer: SpriteLayer, z: number = 0) {
        this.drawSpritesBatched([index], [position], [layer], z)
    }

    public drawSpritesBatched(indices: number[], positions: Vector2[], layers: SpriteLayer[], z: number = 0) {
        const batches: { [key: string]: { indices: number[], positions: Vector3[], layers: SpriteLayer[] } } = {}

        for (let i = 0; i < indices.length; i++) {
            const index = indices[i]
            const position = positions[i]

            for (let k of Object.keys(this.tilesheets)) {
                const tilesheet = this.tilesheets[k]
                if (!tilesheet.isWithinRange(index)) continue

                if (!batches[k]) {
                    batches[k] = {indices: [], positions: [], layers: []}
                }

                batches[k].indices.push(index - tilesheet.range[0]);
                batches[k].layers.push(layers[i])

                const worldY = position.y / TILE_SIZE
                const worldX = position.x / TILE_SIZE

                const newPosition = new Vector3(
                    position.x,
                    position.y,
                    // + 1 to always add an offset because if we didn't, a few sprites would not show up
                    (MAX_DEPTH - MAX_ROW * (worldY + 1)) + worldX + z
                )
                batches[k].positions.push(newPosition)

                break
            }
        }

        for (let k of Object.keys(batches)) {
            const batch = batches[k]
            this.tilesheets[k].drawSpriteLocalIndexBatched(batch.indices, batch.positions, batch.layers)
        }
    }

    public clearAll() {
        for (let k of Object.keys(this.tilesheets)) {
            const tilesheet = this.tilesheets[k]
            tilesheet.clear()
        }
    }

    public removeSprite(position: Vector2, layer: SpriteLayer) {
        for (let k of Object.keys(this.tilesheets)) {
            const tilesheet = this.tilesheets[k]
            tilesheet.removeSpriteAtPosition(position, layer)
        }
    }
}