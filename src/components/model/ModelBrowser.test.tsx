import { mockInvoke } from "../../lib/tauri-mock";
import { render, screen, waitFor } from "../../lib/test-utils";
import { ModelBrowser } from "./ModelBrowser";

describe("ModelBrowser", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("renders model list from IPC", async () => {
    mockInvoke.mockResolvedValue([
      {
        name: "llama-3-8b-q4",
        path: "/models/llama-3-8b-q4.gguf",
        size_bytes: 4_500_000_000,
        blake3_hash: null,
        status: "Available",
      },
    ]);

    render(<ModelBrowser />);

    await waitFor(() => {
      expect(screen.getByTestId("model-llama-3-8b-q4")).toHaveTextContent("llama-3-8b-q4");
      expect(screen.getByTestId("model-llama-3-8b-q4")).toHaveTextContent("4.5 GB");
    });
  });

  it("shows empty state when no models", async () => {
    mockInvoke.mockResolvedValue([]);

    render(<ModelBrowser />);

    await waitFor(() => {
      expect(screen.getByTestId("model-browser")).toHaveTextContent("No models found");
    });
  });
});
