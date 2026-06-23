import { useState } from "react";

interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
}

let messageCounter = 0;
function nextMessageId(): string {
  messageCounter += 1;
  return `msg-${messageCounter}`;
}

export function StreamingChat() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");

  const handleSend = () => {
    const trimmed = input.trim();
    if (!trimmed) return;
    setMessages((prev) => [
      ...prev,
      { id: nextMessageId(), role: "user", content: trimmed },
    ]);
    setInput("");
  };

  return (
    <div data-testid="streaming-chat">
      <div data-testid="message-list">
        {messages.map((m) => (
          <div key={m.id} data-testid={`message-${m.role}`}>
            <strong>{m.role}:</strong> {m.content}
          </div>
        ))}
      </div>
      <input
        data-testid="chat-input"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && handleSend()}
        placeholder="Type a message..."
      />
      <button data-testid="send-button" onClick={handleSend}>
        Send
      </button>
    </div>
  );
}
