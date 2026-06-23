import { render, screen } from "../../lib/test-utils";
import userEvent from "@testing-library/user-event";
import { StreamingChat } from "./StreamingChat";

describe("StreamingChat", () => {
  it("renders empty chat", () => {
    render(<StreamingChat />);
    expect(screen.getByTestId("streaming-chat")).toBeInTheDocument();
    expect(screen.getByTestId("chat-input")).toBeInTheDocument();
  });

  it("sends a message on button click", async () => {
    render(<StreamingChat />);
    const input = screen.getByTestId("chat-input");
    const button = screen.getByTestId("send-button");

    await userEvent.type(input, "Hello world");
    await userEvent.click(button);

    expect(screen.getByTestId("message-user")).toHaveTextContent("Hello world");
  });

  it("clears input after sending", async () => {
    render(<StreamingChat />);
    const input = screen.getByTestId("chat-input") as HTMLInputElement;

    await userEvent.type(input, "Test");
    await userEvent.click(screen.getByTestId("send-button"));

    expect(input.value).toBe("");
  });
});
