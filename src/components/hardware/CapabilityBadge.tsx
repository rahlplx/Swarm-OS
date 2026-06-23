import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface CapabilityScore {
  total: number;
  vram_score: number;
  ram_score: number;
  cpu_score: number;
  backend_bonus: number;
}

export function CapabilityBadge() {
  const [score, setScore] = useState<CapabilityScore | null>(null);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(false);

  const fetchScore = useCallback(() => {
    invoke<CapabilityScore>("get_capability_score")
      .then((s) => {
        setScore(s);
        setError(null);
      })
      .catch((err: unknown) => {
        const msg = String(err);
        setError(msg);
        // Log instead of silently swallowing — helps debugging in dev.
        if (import.meta.env.DEV) {
          console.warn("CapabilityBadge: failed to fetch score:", msg);
        }
      });
  }, []);

  useEffect(() => {
    if (mounted.current) return;
    mounted.current = true;
    queueMicrotask(fetchScore);
  }, [fetchScore]);

  if (error) {
    return (
      <div data-testid="capability-badge" role="alert">
        Score unavailable
      </div>
    );
  }
  if (!score) return null;

  return (
    <div data-testid="capability-badge">
      <strong>Capability Score:</strong> {score.total.toFixed(1)}
    </div>
  );
}
