import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ModelInfo {
  name: string;
  path: string;
  size_bytes: number;
  blake3_hash: string | null;
  status: string;
}

export function ModelBrowser() {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(false);

  const refresh = useCallback(() => {
    invoke<ModelInfo[]>("list_models")
      .then((m) => {
        setModels(m);
        setError(null);
      })
      .catch((err: unknown) => {
        const msg = String(err);
        setError(msg);
        setModels([]);
        if (import.meta.env.DEV) {
          console.warn("ModelBrowser: failed to list models:", msg);
        }
      });
  }, []);

  useEffect(() => {
    if (mounted.current) return;
    mounted.current = true;
    queueMicrotask(refresh);
  }, [refresh]);

  if (error) {
    return (
      <div data-testid="model-browser" role="alert">
        Failed to load models
      </div>
    );
  }

  if (models.length === 0) {
    return <div data-testid="model-browser">No models found</div>;
  }

  return (
    <div data-testid="model-browser">
      <h2>Models</h2>
      <ul>
        {models.map((m) => (
          <li key={m.path} data-testid={`model-${m.name}`}>
            {m.name} — {(m.size_bytes / 1e9).toFixed(1)} GB
          </li>
        ))}
      </ul>
    </div>
  );
}
