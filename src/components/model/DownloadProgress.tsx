interface DownloadProgressProps {
  modelName: string;
  progress: number;
  totalBytes: number;
}

export function DownloadProgress({ modelName, progress, totalBytes }: DownloadProgressProps) {
  const downloadedBytes = totalBytes * (progress / 100);

  return (
    <div data-testid="download-progress">
      <span data-testid="model-name">{modelName}</span>
      <progress value={progress} max={100} data-testid="progress-bar" />
      <span data-testid="progress-text">
        {progress.toFixed(1)}% — {(downloadedBytes / 1e9).toFixed(1)} / {(totalBytes / 1e9).toFixed(1)} GB
      </span>
    </div>
  );
}
