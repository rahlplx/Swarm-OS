import { useEffect, useState } from "react";
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

  const refresh = () => {
    setLoading(true);
    invoke<ModelInfo[]>("list_models")
      .then(setModels)
      .catch(() => setModels([]))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    refresh();
  }, []);

  return { models, loading, refresh };
}
