import { create_spectrum_image } from "../../raytracer/pkg/raytracer";

export function renderImage(width: number, height: number): Uint8ClampedArray {
  const floatImage = create_spectrum_image(width, height);
  return new Uint8ClampedArray(floatImage.map((x) => Math.floor(x * 255)));
}
