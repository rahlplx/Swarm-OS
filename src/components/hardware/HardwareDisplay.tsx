import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { CapabilityBadge } from "./CapabilityBadge";

interface GpuInfo {
  name: string;
  vram_bytes: number;
  backend: string;
}

interface HardwareProfile {
  cpu_cores: number;
  cpu_name: string;
  ram_total_bytes: number;
  ram_available_bytes: number;
  gpus: GpuInfo[];
  os: string;
}

function formatBytes(bytes: number): string {
  const gib = bytes / (1024 * 1024 * 1024);
  return `${gib.toFixed(1)} GiB`;
}

export function HardwareDisplay() {
  const [profile, setProfile] = useState<HardwareProfile | null>(null);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(false);

  const detect = useCallback(() => {
    invoke<HardwareProfile>("detect_hardware")
      .then((p) => {
        setProfile(p);
        setError(null);
      })
      .catch((err: unknown) => setError(String(err)));
  }, []);

  useEffect(() => {
    if (mounted.current) return;
    mounted.current = true;
    queueMicrotask(detect);
  }, [detect]);

  if (error) return <div role="alert">Error: {error}</div>;
  if (!profile) return <div>Detecting hardware...</div>;

  return (
    <div data-testid="hardware-display">
      <h2>Hardware</h2>
      <dl>
        <dt>CPU</dt>
        <dd data-testid="cpu-info">
          {profile.cpu_name} ({profile.cpu_cores} cores)
        </dd>

        <dt>RAM</dt>
        <dd data-testid="ram-info">
          {formatBytes(profile.ram_available_bytes)} / {formatBytes(profile.ram_total_bytes)}
        </dd>

        <dt>GPU</dt>
        <dd data-testid="gpu-info">
          {profile.gpus.length === 0 ? (
            "No GPU detected"
          ) : (
            <ul>
              {profile.gpus.map((gpu, index) => (
                <li key={`${gpu.name}-${index}`}>
                  {gpu.name} — {formatBytes(gpu.vram_bytes)} ({gpu.backend})
                </li>
              ))}
            </ul>
          )}
        </dd>

        <dt>OS</dt>
        <dd data-testid="os-info">{profile.os}</dd>
      </dl>
      <CapabilityBadge />
    </div>
  );
}
