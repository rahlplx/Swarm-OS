interface MessageBubbleProps {
  role: "user" | "assistant";
  content: string;
}

export function MessageBubble({ role, content }: MessageBubbleProps) {
  return (
    <div
      data-testid={`bubble-${role}`}
      style={{
        padding: "0.5rem 1rem",
        margin: "0.25rem 0",
        borderRadius: "8px",
        backgroundColor: role === "user" ? "#6366F1" : "#1A1A1F",
        color: "#fff",
        alignSelf: role === "user" ? "flex-end" : "flex-start",
      }}
    >
      {content}
    </div>
  );
}
