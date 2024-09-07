import { useEffect, useRef } from "react";
import { create_spectrum_image } from "../../raytracer/pkg/raytracer";

function App() {
  const width = 1280;
  const height = 720;
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const image = create_spectrum_image(width, height);
    const canvas = canvasRef.current;
    if (canvas) {
      const ctx = canvas.getContext("2d");
      if (ctx) {
        const imageData = ctx.createImageData(width, height);
        const data = new Uint8ClampedArray(image);
        imageData.data.set(data);
        ctx.putImageData(imageData, 0, 0);
      }
    }
  }, []);

  return (
    <div>
      <canvas ref={canvasRef} width={width} height={height} />
    </div>
  );
}

export default App;
