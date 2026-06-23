import { useEffect, useState } from "react";
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

  useEffect(() => {
    invoke<HardwareProfile>("detect_hardware")
      .then((p) => {
        setProfile(p);
        setLoading(false);
      })
      .catch((err: unknown) => {
        setError(String(err));
        setLoading(false);
      });
  }, []);

  return { profile, loading, error };
}
