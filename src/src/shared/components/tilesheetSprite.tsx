import React, {RefObject, useMemo} from "react"
import {SpritesheetConfig} from "../../tauri/types/spritesheet.js";
import {SlimTilesheet, SlimTilesheets} from "../../features/sprites/slimTilesheets.js";

export type TilesheetSpriteProps = {
    tilesheets: SlimTilesheets,
    spritesheetConfig: RefObject<SpritesheetConfig>
    index: number
    width: number
    height: number
}

const TILES_PER_ROW = 16;

function isWithinRange(tilesheet: SlimTilesheet, index: number) {
    if (!tilesheet.range) return false
    return index >= tilesheet.range[0] && index <= tilesheet.range[1]
}

export function TilesheetSprite(props: TilesheetSpriteProps) {
    if (!props.tilesheets) return <></>;

    const tileInfo = props.spritesheetConfig.current.tile_info[0]

    const {url, range, size} = useMemo(() => {
        for (const key of Object.keys(props.tilesheets.tilesheets)) {
            const tilesheet = props.tilesheets.tilesheets[key]

            const tileEntry = props.spritesheetConfig.current["tiles-new"].find(s => s.file === key)
            const spriteWidth = tileEntry.sprite_width || tileInfo.width
            const spriteHeight = tileEntry.sprite_height || tileInfo.height

            if (isWithinRange(tilesheet, props.index)) {
                return {url: `url(${tilesheet.objectURL})`, range: tilesheet.range, size: [spriteWidth, spriteHeight]}
            }
        }

        return {
            url: `url(${props.tilesheets.fallback.objectURL})`,
            range: [0, 0],
            size: [tileInfo.width, tileInfo.height]
        }
    }, [])

    const backgroundOffsetX = ((props.index - range[0]) % TILES_PER_ROW) * size[0]
    const backgroundOffsetY = Math.floor((props.index - range[0]) / TILES_PER_ROW) * size[1]

    return (
        <div className={"tilesheet-sprite"}
             style={{
                 width: props.width,
                 height: props.height
             }}>
            <div style={{
                backgroundImage: url,
                backgroundPosition: `${-backgroundOffsetX}px ${-backgroundOffsetY}px`,
                backgroundRepeat: "no-repeat",
                transform: `scale(${props.width / size[0]}, ${props.height / size[1]})`,
                transformOrigin: '0 0',
                width: size[0],
                height: size[1],
                imageRendering: "pixelated",
            }}/>
        </div>
    )
}