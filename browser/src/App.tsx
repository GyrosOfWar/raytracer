import { useEffect, useRef } from "react";
import Worker from "./worker?worker";

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

  useEffect(() => {
    callWorker(width, height);
  }, []);

  return (
    <main className="container mx-auto">
      <canvas ref={canvasRef} width={width} height={height} />
    </main>
  );
}

export default App;
