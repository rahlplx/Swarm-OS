import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface HardwareProfile {
  cpu_cores: number;
  cpu_name: string;
  ram_total_bytes: number;
  ram_available_bytes: number;
  gpus: Array<{ name: string; vram_bytes: number; backend: string }>;
  os: string;
}

export function useHardware() {
  const [profile, setProfile] = useState<HardwareProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(false);

  const detect = useCallback(() => {
    setLoading(true);
    invoke<HardwareProfile>("detect_hardware")
      .then((p) => {
        setProfile(p);
        setError(null);
      })
      .catch((err: unknown) => {
        setError(String(err));
        setProfile(null);
      })
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    // Defer to microtask to avoid setState-during-effect lint warning.
    if (mounted.current) return;
    mounted.current = true;
    queueMicrotask(detect);
  }, [detect]);

  return { profile, loading, error, refresh: detect };
}
