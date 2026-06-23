interface DownloadProgressProps {
  modelName: string;
  progress: number;
  totalBytes: number;
}

function clamp(value: number, min: number, max: number): number {
  if (Number.isNaN(value)) return min;
  return Math.min(Math.max(value, min), max);
}

export function DownloadProgress({ modelName, progress, totalBytes }: DownloadProgressProps) {
  const safeProgress = clamp(progress, 0, 100);
  const downloadedBytes = totalBytes * (safeProgress / 100);

  return (
    <div data-testid="download-progress">
      <span data-testid="model-name">{modelName}</span>
      <progress value={safeProgress} max={100} data-testid="progress-bar" />
      <span data-testid="progress-text">
        {safeProgress.toFixed(1)}% — {(downloadedBytes / 1e9).toFixed(1)} / {(totalBytes / 1e9).toFixed(1)} GB
      </span>
    </div>
  );
}
