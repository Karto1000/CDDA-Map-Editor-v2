export type CDDAIdentifier = string;

export type ParameterIdentifier = string;

export interface Switch {
    param: ParameterIdentifier;
    fallback: CDDAIdentifier;
}

export type MeabyVec<T> = T | T[];

export function meabyVecToArray<T>(meabyVec: MeabyVec<T>): T[] {
    if (Array.isArray(meabyVec)) return meabyVec
    return [meabyVec]
}

export interface Weighted<T> {
    data: T;
    weight: number;
}

export type MeabyWeighted<T> = T | Weighted<T>;

export function meabyWeightedToData<T>(meabyWeighted: MeabyWeighted<T>): T {
    if (typeof meabyWeighted === "string") return meabyWeighted
    if (typeof meabyWeighted === "object") {
        if ("data" in meabyWeighted && "weight" in meabyWeighted) return meabyWeighted.data
    }
}

export function meabyWeightedToWeighted<T>(meabyWeighted: MeabyWeighted<T>): Weighted<T> {
    if (typeof meabyWeighted === "object") {
        if ("data" in meabyWeighted && "weight" in meabyWeighted) return meabyWeighted
    }
    return {data: meabyWeightedToData(meabyWeighted), weight: 1}
}

export type CDDADistributionInner =
    | CDDAIdentifier
    | {
    param: ParameterIdentifier;
    fallback?: CDDAIdentifier;
}
    | {
    switch: Switch;
    cases: Record<CDDAIdentifier, CDDAIdentifier>;
}
    | {
    distribution: MeabyVec<MeabyWeighted<CDDAIdentifier>>;
};

export type MapGenValue =
    | CDDAIdentifier
    | {
    param: ParameterIdentifier;
    fallback?: CDDAIdentifier;
}
    | {
    switch: Switch;
    cases: Record<CDDAIdentifier, CDDAIdentifier>;
}
    | MeabyVec<MeabyWeighted<CDDADistributionInner>>;


export type MapData = {
    palettes: MapGenValue[]
}

export type StaticSprite = {
    position: string
    index: number
    layer: number
    rotate_deg: number
    z: number
}

export type AnimatedSprite = {
    position: string
    indices: number[],
    layer: number
    rotate_deg: number
    z: number,
}

export type FallbackSprite = {
    position: string,
    index: number
    z: number
}