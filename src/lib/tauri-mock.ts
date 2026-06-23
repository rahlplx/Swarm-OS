import { vi } from "vitest";

const mockInvoke = vi.fn();
const mockChannel = { onmessage: vi.fn() };

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
  Channel: vi.fn(() => mockChannel),
}));

export { mockInvoke, mockChannel };
