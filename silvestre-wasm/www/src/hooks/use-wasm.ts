import { useState, useEffect } from "react";
import init from "silvestre-wasm";
import wasmUrl from "silvestre-wasm/silvestre_wasm_bg.wasm?url";

type WasmStatus = "loading" | "ready" | "error";

export function useWasm() {
  const [status, setStatus] = useState<WasmStatus>("loading");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    init({ module_or_path: wasmUrl })
      .then(() => {
        if (!cancelled) setStatus("ready");
      })
      .catch((e: unknown) => {
        if (!cancelled) {
          setStatus("error");
          setError(e instanceof Error ? e.message : String(e));
        }
      });

    return () => { cancelled = true; };
  }, []);

  return { status, error };
}
