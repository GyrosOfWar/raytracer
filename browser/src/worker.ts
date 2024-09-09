import { renderImage } from "./render";

console.log("worker loaded");

self.onmessage = (event: MessageEvent<[number, number]>) => {
  console.log("worker received message");
  const [w, h] = event.data;
  const image = renderImage(w, h);
  self.postMessage(image);
};

export {};
