import { renderHook, act } from "@testing-library/react";
import { useInference } from "./useInference";

describe("useInference", () => {
  it("starts in idle state", () => {
    const { result } = renderHook(() => useInference());
    expect(result.current.generating).toBe(false);
    expect(result.current.tokens).toEqual([]);
    expect(result.current.error).toBeNull();
  });

  it("returns placeholder error on generate", () => {
    const { result } = renderHook(() => useInference());

    act(() => {
      result.current.generate("hello");
    });

    expect(result.current.error).toContain("placeholder");
  });

  it("resets state", () => {
    const { result } = renderHook(() => useInference());

    act(() => {
      result.current.generate("test");
    });

    act(() => {
      result.current.reset();
    });

    expect(result.current.error).toBeNull();
    expect(result.current.tokens).toEqual([]);
  });
});
