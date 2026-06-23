interface TrayStatusProps {
  status: "idle" | "running" | "paused";
  tokensToday: number;
  creditsEarned: number;
}

export function TrayStatus({ status, tokensToday, creditsEarned }: TrayStatusProps) {
  const statusColors = {
    idle: "#6B7280",
    running: "#22C55E",
    paused: "#EAB308",
  };

  return (
    <div data-testid="tray-status">
      <span
        data-testid="status-indicator"
        style={{ color: statusColors[status] }}
      >
        {status.toUpperCase()}
      </span>
      <span data-testid="tokens-today">{tokensToday.toLocaleString()} tokens today</span>
      <span data-testid="credits-earned">{creditsEarned.toFixed(2)} credits</span>
    </div>
  );
}
