import {Tilesheet} from "./tilesheet.ts";
import {Vector2} from "three";

export class Tilesheets {
    public tilesheets: { [name: string]: Tilesheet }

    constructor(tilesheets: { [name: string]: Tilesheet }) {
        this.tilesheets = tilesheets
    }

    public drawSprite(index: number, position: Vector2) {
        Object.keys(this.tilesheets).forEach(k => {
            const tilesheet = this.tilesheets[k]
            if (!tilesheet.isWithinRange(index)) return

            tilesheet.drawSpriteLocalIndex(index - tilesheet.range[0], position)
        })
    }
}