import { mockInvoke } from "../../lib/tauri-mock";
import { render, screen, waitFor } from "../../lib/test-utils";
import { HardwareDisplay } from "./HardwareDisplay";

describe("HardwareDisplay", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("renders hardware profile from Tauri IPC", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "detect_hardware") {
        return Promise.resolve({
          cpu_cores: 8,
          cpu_name: "Test CPU i9",
          ram_total_bytes: 32 * 1024 * 1024 * 1024,
          ram_available_bytes: 16 * 1024 * 1024 * 1024,
          gpus: [
            {
              name: "RTX 4090",
              vram_bytes: 24 * 1024 * 1024 * 1024,
              backend: "Cuda",
            },
          ],
          os: "Linux 6.8",
        });
      }
      if (cmd === "get_capability_score") {
        return Promise.resolve({
          total: 126.0,
          vram_score: 96.0,
          ram_score: 16.0,
          cpu_score: 4.0,
          backend_bonus: 10.0,
        });
      }
      return Promise.reject("unknown command");
    });

    render(<HardwareDisplay />);

    await waitFor(() => {
      expect(screen.getByTestId("cpu-info")).toHaveTextContent("Test CPU i9");
      expect(screen.getByTestId("cpu-info")).toHaveTextContent("8 cores");
    });

    expect(screen.getByTestId("gpu-info")).toHaveTextContent("RTX 4090");
    expect(screen.getByTestId("os-info")).toHaveTextContent("Linux 6.8");
  });

  it("shows no GPU when none detected", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "detect_hardware") {
        return Promise.resolve({
          cpu_cores: 4,
          cpu_name: "i5",
          ram_total_bytes: 8 * 1024 * 1024 * 1024,
          ram_available_bytes: 4 * 1024 * 1024 * 1024,
          gpus: [],
          os: "Windows",
        });
      }
      if (cmd === "get_capability_score") {
        return Promise.resolve({
          total: 5.0,
          vram_score: 0,
          ram_score: 4.0,
          cpu_score: 1.0,
          backend_bonus: 0,
        });
      }
      return Promise.reject("unknown command");
    });

    render(<HardwareDisplay />);

    await waitFor(() => {
      expect(screen.getByTestId("gpu-info")).toHaveTextContent("No GPU detected");
    });
  });

  it("shows error on IPC failure", async () => {
    mockInvoke.mockRejectedValue("connection failed");

    render(<HardwareDisplay />);

    await waitFor(() => {
      expect(screen.getByRole("alert")).toHaveTextContent("Error: connection failed");
    });
  });
});
