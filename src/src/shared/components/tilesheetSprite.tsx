import React, {RefObject, useMemo} from "react"
import {Tilesheets} from "../../features/sprites/tilesheets.js";
import {SpritesheetConfig} from "../../tauri/types/spritesheet.js";

export type TilesheetSpriteProps = {
    tilesheets: RefObject<Tilesheets>
    spritesheetConfig: RefObject<SpritesheetConfig>
    index: number
    width: number
    height: number
}

const TILES_PER_ROW = 16;

export function TilesheetSprite(props: TilesheetSpriteProps) {
    if (!props.tilesheets.current) return <></>;

    const tileInfo = props.spritesheetConfig.current.tile_info[0]

    const {url, range} = useMemo(() => {
        for (const key of Object.keys(props.tilesheets.current.tilesheets)) {
            const tilesheet = props.tilesheets.current.tilesheets[key]
            console.log(tilesheet.range, tilesheet.spritesheetInfo, props.index, tilesheet.isWithinRange(props.index))

            if (tilesheet.isWithinRange(props.index)) {
                return {url: `url(${tilesheet.objectURL})`, range: tilesheet.range}
            }
        }

        return {
            url: `url(${props.tilesheets.current.fallback.objectURL})`,
            range: [0, 0]
        }
    }, [])

    const backgroundOffsetX = ((props.index - range[0]) % TILES_PER_ROW) * tileInfo.width
    const backgroundOffsetY = Math.floor((props.index - range[0]) / TILES_PER_ROW) * tileInfo.height

    return (
        <div className={"tilesheet-sprite"} style={{
            width: props.width,
            height: props.height
        }}>
            <div style={{
                backgroundImage: url,
                backgroundPosition: `${-backgroundOffsetX}px ${-backgroundOffsetY}px`,
                backgroundRepeat: "no-repeat",
                transform: `scale(${props.width / tileInfo.width}, ${props.height / tileInfo.height})`,
                transformOrigin: '0 0',
                width: tileInfo.width,
                height: tileInfo.height,
                imageRendering: "pixelated",
            }}/>
        </div>
    )
}