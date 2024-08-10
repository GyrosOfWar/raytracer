import init, { add } from "raytracer";
import { useEffect, useState } from "react";

function App() {
  const [result, setResult] = useState<number | null>(null);
  useEffect(() => {
    init().then(() => {
      setResult(add(16, 12));
    });
  });

  return (
    <div>
      <p>Result: {result}</p>
    </div>
  );
}

export default App;
