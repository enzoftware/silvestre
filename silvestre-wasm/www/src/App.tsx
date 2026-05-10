import { useState, useCallback } from "react";
import type { WasmImage } from "silvestre-wasm";
import { useWasm } from "@/hooks/use-wasm";
import { useFilter } from "@/hooks/use-filter";
import { ImageUpload } from "@/components/image-upload";
import { ImageComparison } from "@/components/image-comparison";
import { FilterPanel } from "@/components/filter-panel";
import { Skeleton } from "@/components/ui/skeleton";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";

export default function App() {
  const { status, error } = useWasm();
  const { processing, applyFilter } = useFilter();
  const [original, setOriginal] = useState<WasmImage | null>(null);
  const [filtered, setFiltered] = useState<WasmImage | null>(null);

  const handleImageLoaded = useCallback((image: WasmImage) => {
    setOriginal(image);
    setFiltered(null);
  }, []);

  const handleApply = useCallback(
    async (filterName: string, params: Record<string, unknown>) => {
      if (!original) return;
      try {
        const source = filtered ?? original;
        const result = await applyFilter(source, filterName, params);
        setFiltered(result);
      } catch (e) {
        console.error("Filter error:", e);
      }
    },
    [original, filtered, applyFilter],
  );

  const handleReset = useCallback(() => {
    setFiltered(null);
  }, []);

  const handleDownload = useCallback(() => {
    const image = filtered ?? original;
    if (!image) return;
    const bytes = image.toBytes("png");
    const blob = new Blob([bytes.slice().buffer as ArrayBuffer], {
      type: "image/png",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "silvestre-output.png";
    a.click();
    URL.revokeObjectURL(url);
  }, [filtered, original]);

  if (status === "loading") {
    return (
      <div className="min-h-screen bg-background p-8">
        <div className="max-w-5xl mx-auto space-y-6">
          <Skeleton className="h-10 w-64" />
          <Skeleton className="h-4 w-96" />
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="md:col-span-2">
              <Skeleton className="h-64 w-full" />
            </div>
            <Skeleton className="h-64 w-full" />
          </div>
        </div>
      </div>
    );
  }

  if (status === "error") {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center p-8">
        <div className="text-center space-y-2">
          <h1 className="text-2xl font-bold text-destructive">
            Failed to load WASM module
          </h1>
          <p className="text-muted-foreground">{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b border-border px-8 py-4">
        <div className="max-w-5xl mx-auto flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold tracking-tight">silvestre</h1>
            <p className="text-sm text-muted-foreground">
              Image processing in the browser, powered by Rust + WebAssembly
            </p>
          </div>
          {original && (
            <div className="flex gap-2">
              {filtered && (
                <Button variant="outline" size="sm" onClick={handleReset}>
                  Reset
                </Button>
              )}
              <Button variant="outline" size="sm" onClick={handleDownload}>
                Download PNG
              </Button>
            </div>
          )}
        </div>
      </header>

      <main className="max-w-5xl mx-auto p-8 space-y-6">
        {!original ? (
          <ImageUpload onImageLoaded={handleImageLoaded} />
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="md:col-span-2 space-y-4">
              <ImageComparison
                original={original}
                filtered={filtered}
                processing={processing}
              />
            </div>
            <div className="space-y-4">
              <FilterPanel disabled={processing} onApply={handleApply} />
              <Separator />
              <Button
                variant="ghost"
                className="w-full text-muted-foreground"
                onClick={() => {
                  setOriginal(null);
                  setFiltered(null);
                }}
              >
                Load different image
              </Button>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
