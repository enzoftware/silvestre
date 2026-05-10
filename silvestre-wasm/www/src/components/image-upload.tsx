import { useCallback, useRef, useState } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { WasmImage } from "silvestre-wasm";

interface ImageUploadProps {
  onImageLoaded: (image: WasmImage) => void;
}

export function ImageUpload({ onImageLoaded }: ImageUploadProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);

  const loadFile = useCallback(
    async (file: File) => {
      const buffer = await file.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      const image = WasmImage.loadFromBytes(bytes);
      onImageLoaded(image);
    },
    [onImageLoaded]
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(false);
      const file = e.dataTransfer.files[0];
      if (file) loadFile(file);
    },
    [loadFile]
  );

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) loadFile(file);
    },
    [loadFile]
  );

  return (
    <Card
      className={`border-2 border-dashed transition-colors ${
        dragOver ? "border-primary bg-primary/5" : "border-border"
      }`}
      onDragOver={(e) => {
        e.preventDefault();
        setDragOver(true);
      }}
      onDragLeave={() => setDragOver(false)}
      onDrop={handleDrop}
    >
      <CardContent className="flex flex-col items-center justify-center py-12 gap-4">
        <p className="text-muted-foreground text-sm">
          Drag & drop an image here, or click to browse
        </p>
        <Button variant="outline" onClick={() => inputRef.current?.click()}>
          Choose File
        </Button>
        <input
          ref={inputRef}
          type="file"
          accept="image/png,image/jpeg,image/bmp"
          className="hidden"
          onChange={handleChange}
        />
        <p className="text-xs text-muted-foreground">
          Supports PNG, JPEG, BMP
        </p>
      </CardContent>
    </Card>
  );
}
