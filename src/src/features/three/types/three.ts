import {AmbientLight, GridHelper, OrthographicCamera, Raycaster, Scene, WebGLRenderer} from "three";
import {ArcballControls} from "three/examples/jsm/controls/ArcballControls.js";
import {MutableRefObject, RefObject} from "react";
import {OrbitControls} from "three/examples/jsm/controls/OrbitControls.js";

export type Canvas = {
    canvasRef: RefObject<HTMLCanvasElement>,
    canvasContainerRef: RefObject<HTMLDivElement>
}

export type ThreeConfig = {
    scene: Scene,
    raycaster: Raycaster
    stats: Stats
    camera: OrthographicCamera
    renderer: WebGLRenderer
    controls: OrbitControls
    ambientLight: AmbientLight
    gridHelper: GridHelper
}