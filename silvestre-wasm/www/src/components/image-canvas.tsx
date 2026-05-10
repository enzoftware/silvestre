import { useRef, useEffect } from "react";
import type { WasmImage } from "silvestre-wasm";

interface ImageCanvasProps {
  image: WasmImage | null;
  className?: string;
}

export function ImageCanvas({ image, className }: ImageCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !image) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const imageData = image.toImageData();
    canvas.width = imageData.width;
    canvas.height = imageData.height;
    ctx.putImageData(imageData, 0, 0);
  }, [image]);

  if (!image) return null;

  return (
    <canvas
      ref={canvasRef}
      className={className}
      style={{ maxWidth: "100%", height: "auto" }}
    />
  );
}
