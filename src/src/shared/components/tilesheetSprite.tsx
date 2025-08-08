import React, {RefObject, useMemo} from "react"
import {SpritesheetConfig, TileNew} from "../../tauri/types/spritesheet.js";
import {SlimTilesheet, SlimTilesheets} from "../../features/sprites/slimTilesheets.js";
import {clsx} from "clsx";

export type TilesheetSpriteProps = {
    tilesheets: SlimTilesheets,
    spritesheetConfig: RefObject<SpritesheetConfig>
    index: number
    scale: number
    isFallback?: boolean
    className?: string
}

const TILES_PER_ROW = 16;

function isWithinRange(tilesheet: SlimTilesheet, index: number) {
    if (!tilesheet.range) return false
    return index >= tilesheet.range[0] && index <= tilesheet.range[1]
}

export function TilesheetSprite(props: TilesheetSpriteProps) {
    if (!props.tilesheets) return <></>;

    const tileInfo = props.spritesheetConfig.current.tile_info[0]

    const {url, range, size, offset} = useMemo(() => {
        if (!props.isFallback) {
            for (const key of Object.keys(props.tilesheets.tilesheets)) {
                const tilesheet = props.tilesheets.tilesheets[key]

                const tileEntry = props.spritesheetConfig.current["tiles-new"].find(s => s.file === key) as TileNew
                const spriteWidth = tileEntry.sprite_width || tileInfo.width
                const spriteHeight = tileEntry.sprite_height || tileInfo.height
                const spriteOffsetX = -tileEntry.sprite_offset_x || 0
                const spriteOffsetY = -tileEntry.sprite_offset_y || 0

                if (isWithinRange(tilesheet, props.index)) {
                    return {
                        url: `url(${tilesheet.objectURL})`,
                        range: tilesheet.range,
                        size: [spriteWidth, spriteHeight],
                        offset: [
                            spriteOffsetX,
                            spriteOffsetY
                        ]
                    }
                }
            }
        }

        return {
            url: `url(${props.tilesheets.fallback.objectURL})`,
            range: [0, 0],
            size: [tileInfo.width, tileInfo.height],
            offset: [0, 0]
        }
    }, [])

    const backgroundOffsetX = ((props.index - range[0]) % TILES_PER_ROW) * size[0]
    const backgroundOffsetY = Math.floor((props.index - range[0]) / TILES_PER_ROW) * size[1]

    return (
        <div className={clsx("tilesheet-sprite", props.className)}
             style={{
                 width: 64,
                 height: 64,
             }}>
            <div style={{
                position: "absolute",
                top: -offset[1] * props.scale,
                left: -offset[0] * props.scale,
                backgroundImage: url,
                backgroundPosition: `${-backgroundOffsetX}px ${-backgroundOffsetY}px`,
                backgroundRepeat: "no-repeat",
                transform: `scale(${props.scale}, ${props.scale})`,
                transformOrigin: `top left`,
                // transformOrigin: `${-backgroundOffsetX - offset[0]}px ${-backgroundOffsetY - offset[1]}px`,
                width: size[0],
                height: size[1],
                imageRendering: "pixelated",
            }}/>
        </div>
    )
}