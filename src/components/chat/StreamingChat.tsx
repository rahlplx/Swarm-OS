import { useState } from "react";

interface Message {
  role: "user" | "assistant";
  content: string;
}

export function StreamingChat() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");

  const handleSend = () => {
    if (!input.trim()) return;
    setMessages((prev) => [...prev, { role: "user", content: input }]);
    setInput("");
  };

  return (
    <div data-testid="streaming-chat">
      <div data-testid="message-list">
        {messages.map((m, i) => (
          <div key={i} data-testid={`message-${m.role}`}>
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
