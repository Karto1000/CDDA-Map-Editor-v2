#version 300 es

in vec2 vUV;

out vec4 fragColor;

uniform vec2 uScreenSize;
uniform vec2 uOffset;
uniform vec3 uLightest;
uniform vec3 uDarker;
uniform int uCellSize;

void main() {
    bool isXDivisibleBy32 = int(vUV.x * uScreenSize.x + uOffset.x) % uCellSize == 0;
    bool isYDivisibleBy32 = int(vUV.y * uScreenSize.y + uOffset.y) % uCellSize == 0;

    if (isXDivisibleBy32 || isYDivisibleBy32) {
        fragColor = vec4(uLightest.rgb, .1);
    } else {
        fragColor = vec4(uDarker.rgb, 1.);
    }
}