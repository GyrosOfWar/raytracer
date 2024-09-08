import { useEffect, useRef, useState } from "react";
import { create_spectrum_image } from "../../raytracer/pkg/raytracer";

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

function App() {
  const width = 1280;
  const height = 720;
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [state, setState] = useState<State>("initializing");
  const [numSamples, setNumSamples] = useState(0);
  const buffer = useRef(new Float32Array(width * height * 4));

  useEffect(() => {
    setState("rendering");
    const newSamples = create_spectrum_image(width, height);
    setNumSamples((n) => n + 1);
    buffer.current = add(buffer.current, newSamples);

    const canvas = canvasRef.current;
    if (canvas) {
      const ctx = canvas.getContext("2d");
      if (ctx) {
        const imageData = ctx.createImageData(width, height);
        const data = toImageBuffer(buffer.current, numSamples);
        imageData.data.set(data);
        ctx.putImageData(imageData, 0, 0);
        setState("done");
      }
    }
  }, []);

  return (
    <main className="container mx-auto">
      <p className="my-4">{state}</p>

      <canvas ref={canvasRef} width={width} height={height} />
    </main>
  );
}

export default App;
