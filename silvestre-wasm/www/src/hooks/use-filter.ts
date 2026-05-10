import { useState, useCallback } from "react";
import type { WasmImage } from "silvestre-wasm";

export function useFilter() {
  const [processing, setProcessing] = useState(false);

  const applyFilter = useCallback(
    async (
      image: WasmImage,
      filterName: string,
      params: Record<string, unknown>
    ): Promise<WasmImage> => {
      setProcessing(true);
      // Yield to UI before blocking WASM call
      await new Promise((r) => setTimeout(r, 0));
      try {
        const result = image.applyFilter(filterName, params);
        return result;
      } finally {
        setProcessing(false);
      }
    },
    []
  );

  return { processing, applyFilter };
}
