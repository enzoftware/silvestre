import { ImageCanvas } from "./image-canvas";
import { Badge } from "@/components/ui/badge";
import type { WasmImage } from "silvestre-wasm";

interface ImageComparisonProps {
  original: WasmImage | null;
  filtered: WasmImage | null;
  processing: boolean;
}

export function ImageComparison({
  original,
  filtered,
  processing,
}: ImageComparisonProps) {
  if (!original) return null;

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div className="space-y-2">
        <Badge variant="outline">Original</Badge>
        <ImageCanvas
          image={original}
          className="rounded-lg border border-border"
        />
        {original && (
          <p className="text-xs text-muted-foreground">
            {original.width} x {original.height}
          </p>
        )}
      </div>
      <div className="space-y-2">
        <Badge variant="outline">
          {processing ? "Processing..." : "Result"}
        </Badge>
        <div className="relative">
          {processing && (
            <div className="absolute inset-0 flex items-center justify-center bg-background/80 rounded-lg z-10">
              <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
            </div>
          )}
          <ImageCanvas
            image={filtered ?? original}
            className="rounded-lg border border-border"
          />
          {(filtered ?? original) && (
            <p className="text-xs text-muted-foreground">
              {(filtered ?? original)!.width} x{" "}
              {(filtered ?? original)!.height}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
