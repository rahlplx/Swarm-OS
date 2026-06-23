import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ModelInfo {
  name: string;
  path: string;
  size_bytes: number;
  blake3_hash: string | null;
  status: string;
}

export function useModelManager() {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const mounted = useRef(false);

  const refresh = useCallback(() => {
    setLoading(true);
    invoke<ModelInfo[]>("list_models")
      .then(setModels)
      .catch(() => setModels([]))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    // Avoid calling setState synchronously during the effect's commit phase.
    // Defer to a microtask so React's render-loop invariant isn't tripped.
    if (mounted.current) return;
    mounted.current = true;
    queueMicrotask(refresh);
  }, [refresh]);

  return { models, loading, refresh };
}
