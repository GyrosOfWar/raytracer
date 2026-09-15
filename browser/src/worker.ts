import { renderImage } from "./render";
import initRaytracer from "raytracer";

const wasmReady = initRaytracer();

self.onmessage = async (event: MessageEvent<[number, number]>) => {
	await wasmReady;
	const [w, h] = event.data;
	const image = renderImage(w, h);
	self.postMessage(image);
};

export {};
