precision mediump float;

uniform vec2 uScreenSize;
uniform vec2 uOffset;
uniform vec3 uLightest;
uniform vec3 uDarker;
uniform int uCellSize;
uniform int uZoom;

vec4 to_linear(vec4 nonlinear_color) {
    vec4 cutoff = step(nonlinear_color, vec4(0.04045));
    vec4 higher = pow((nonlinear_color + vec4(0.055)) / vec4(1.055), vec4(2.4));
    vec4 lower = nonlinear_color / vec4(12.92);
    return mix(higher, lower, cutoff);
}

void main() {
    bool isXDivisibleBy32 = int(gl_FragCoord.x + uOffset.x) % uCellSize == 0;
    bool isYDivisibleBy32 = int(gl_FragCoord.y + uOffset.y) % uCellSize == 0;

    if (isXDivisibleBy32 || isYDivisibleBy32) {
        gl_FragColor = to_linear(vec4(uLightest.rgb, .1));
    } else {
        gl_FragColor = to_linear(vec4(uDarker.rgb, 1.));
    }
}