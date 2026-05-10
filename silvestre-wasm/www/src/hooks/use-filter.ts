import { useState, useCallback, useRef } from "react";
import type { WasmImage } from "silvestre-wasm";

export function useFilter() {
  const [processing, setProcessing] = useState(false);
  const inFlight = useRef(0);

  const applyFilter = useCallback(
    async (
      image: WasmImage,
      filterName: string,
      params: Record<string, unknown>
    ): Promise<WasmImage> => {
      inFlight.current += 1;
      setProcessing(true);
      // Yield to UI before blocking WASM call
      await new Promise((r) => setTimeout(r, 0));
      try {
        const result = image.applyFilter(filterName, params);
        return result;
      } finally {
        inFlight.current -= 1;
        if (inFlight.current === 0) setProcessing(false);
      }
    },
    []
  );

  return { processing, applyFilter };
}
