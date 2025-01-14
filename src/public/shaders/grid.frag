#version 300 es

in vec2 vUV;

out vec4 fragColor;

uniform vec2 uScreenSize;
uniform vec2 uOffset;

vec3 dark = vec3(.07, .07, .07);
vec3 white = vec3(1., 1., 1.);

void main() {
    bool isXDivisibleBy32 = int(vUV.x * uScreenSize.x + uOffset.x) % 32 == 0;
    bool isYDivisibleBy32 = int(vUV.y * uScreenSize.y + uOffset.y) % 32 == 0;

    if (isXDivisibleBy32 || isYDivisibleBy32) {
        fragColor = vec4(white.xyz, .1);
    } else {
        fragColor = vec4(dark.xyz, 1.);
    }
}