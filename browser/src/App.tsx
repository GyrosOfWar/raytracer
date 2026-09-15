import { useEffect, useRef } from "react";
import Worker from "./worker?worker";

function callWorker(width: number, height: number): Promise<Uint8ClampedArray> {
	return new Promise((resolve) => {
		const worker = new Worker();

		worker.postMessage([width, height]);

		worker.addEventListener("message", (event) => {
			const array: Uint8ClampedArray = event.data;
			resolve(array);
		});
	});
}

function App() {
	const width = 1280;
	const height = 720;
	const canvasRef = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		callWorker(width, height).then((bytes) => {
			if (canvasRef.current) {
				const ctx = canvasRef.current.getContext("2d");
				if (ctx) {
					const imageData = new ImageData(bytes, width, height);
					ctx.putImageData(imageData, 0, 0);
				}
			}
		});
	}, []);

	return (
		<main className="container mx-auto">
			<canvas ref={canvasRef} width={width} height={height} />
		</main>
	);
}

export default App;
