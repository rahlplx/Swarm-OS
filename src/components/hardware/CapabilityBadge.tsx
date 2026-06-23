import { useEffect, useState } from "react";
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

  useEffect(() => {
    invoke<CapabilityScore>("get_capability_score")
      .then(setScore)
      .catch(() => {});
  }, []);

  if (!score) return null;

  return (
    <div data-testid="capability-badge">
      <strong>Capability Score:</strong> {score.total.toFixed(1)}
    </div>
  );
}
