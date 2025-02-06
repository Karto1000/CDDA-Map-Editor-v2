export class Shaders
{
    public static get vertexShader()
    {
        return (
            `precision highp float;

uniform mat4 modelViewMatrix;
uniform mat4 projectionMatrix;

attribute vec3 position;
attribute vec2 uv;
attribute mat4 instanceMatrix;
attribute vec2 uvOffset;

uniform vec2 atlasSize;

varying vec2 vUv;

void main()
{
	vUv = uvOffset + (uv / atlasSize);
	gl_Position = projectionMatrix * modelViewMatrix * instanceMatrix * vec4(position, 1.0);
}
`);
    }

    public static get fragmentShader()
    {
        return (
            `precision highp float;

varying vec2 vUv;

uniform sampler2D map;

void main()
{
	gl_FragColor = texture2D(map, vUv);
}
`);
    }
}