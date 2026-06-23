import { mockInvoke } from "../lib/tauri-mock";
import { renderHook, waitFor } from "@testing-library/react";
import { useHardware } from "./useHardware";

describe("useHardware", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("returns hardware profile on success", async () => {
    mockInvoke.mockResolvedValue({
      cpu_cores: 8,
      cpu_name: "Test CPU",
      ram_total_bytes: 16e9,
      ram_available_bytes: 8e9,
      gpus: [],
      os: "Linux",
    });

    const { result } = renderHook(() => useHardware());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
      expect(result.current.profile?.cpu_cores).toBe(8);
      expect(result.current.error).toBeNull();
    });
  });

  it("returns error on failure", async () => {
    mockInvoke.mockRejectedValue("IPC failed");

    const { result } = renderHook(() => useHardware());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBe("IPC failed");
      expect(result.current.profile).toBeNull();
    });
  });
});
