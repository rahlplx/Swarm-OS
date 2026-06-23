import { useEffect, useState } from "react";
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

  useEffect(() => {
    invoke<ModelInfo[]>("list_models")
      .then(setModels)
      .catch(() => {});
  }, []);

  if (models.length === 0) {
    return <div data-testid="model-browser">No models found</div>;
  }

  return (
    <div data-testid="model-browser">
      <h2>Models</h2>
      <ul>
        {models.map((m) => (
          <li key={m.name} data-testid={`model-${m.name}`}>
            {m.name} — {(m.size_bytes / 1e9).toFixed(1)} GB
          </li>
        ))}
      </ul>
    </div>
  );
}
