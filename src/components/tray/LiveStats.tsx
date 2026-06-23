interface LiveStatsProps {
  tokensPerSecond: number;
  activeJobs: number;
  vramUsedPercent: number;
}

export function LiveStats({ tokensPerSecond, activeJobs, vramUsedPercent }: LiveStatsProps) {
  return (
    <div data-testid="live-stats">
      <div data-testid="tps">{tokensPerSecond.toFixed(1)} tok/s</div>
      <div data-testid="active-jobs">{activeJobs} active jobs</div>
      <div data-testid="vram-usage">{vramUsedPercent.toFixed(0)}% VRAM</div>
    </div>
  );
}
