import {Tilesheet} from "./tilesheet.ts";
import {Vector2} from "three";

export class Tilesheets {
    public tilesheets: { [name: string]: Tilesheet }

    constructor(tilesheets: { [name: string]: Tilesheet }) {
        this.tilesheets = tilesheets
    }

    public drawSprite(index: number, position: Vector2) {
        this.drawSpritesBatched([index], [position])
    }

    public drawSpritesBatched(indices: number[], positions: Vector2[]) {
        const batches: { [key: string]: { indices: number[], positions: Vector2[] } } = {}

        for (let i = 0; i < indices.length; i++) {
            const index = indices[i]
            const position = positions[i]

            for (let k of Object.keys(this.tilesheets)) {
                const tilesheet = this.tilesheets[k]
                if (!tilesheet.isWithinRange(index)) continue

                if (!batches[k]) {
                    batches[k] = {indices: [], positions: []}
                }

                batches[k].indices.push(index - tilesheet.range[0]);
                batches[k].positions.push(position)
                break
            }
        }

        for (let k of Object.keys(batches)) {
            const batch = batches[k]
            this.tilesheets[k].drawSpriteLocalIndexBatched(batch.indices, batch.positions)
        }
    }
}