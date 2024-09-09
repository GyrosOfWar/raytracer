import { useEffect, useRef, useState } from "react";
import Worker from "./worker?worker";

type State = "initializing" | "rendering" | "done";

function add(a: Float32Array, b: Float32Array): Float32Array {
  return a.map((_, i) => a[i] + b[i]);
}

function toImageBuffer(
  buffer: Float32Array,
  numSamples: number
): Uint8ClampedArray {
  return new Uint8ClampedArray(
    buffer.map((x) => Math.floor((x / numSamples) * 255))
  );
}

function callWorker(width: number, height: number) {
  const worker = new Worker();

  worker.postMessage([width, height]);

  worker.addEventListener("message", (event) => {
    console.log(event);
  });
}

function App() {
  const width = 1280;
  const height = 720;
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [state, setState] = useState<State>("initializing");
  const [numSamples, setNumSamples] = useState(0);
  const buffer = useRef(new Float32Array(width * height * 4));

  useEffect(() => {
    setState("rendering");
    callWorker(width, height);
  }, []);

  return (
    <main className="container mx-auto">
      <p className="my-4">Samples: {numSamples}</p>

      <canvas ref={canvasRef} width={width} height={height} />
    </main>
  );
}

export default App;
