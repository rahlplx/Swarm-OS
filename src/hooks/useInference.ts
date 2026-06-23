import { useState, useCallback } from "react";

interface InferenceState {
  generating: boolean;
  tokens: string[];
  tokensPerSec: number;
  error: string | null;
}

export function useInference() {
  const [state, setState] = useState<InferenceState>({
    generating: false,
    tokens: [],
    tokensPerSec: 0,
    error: null,
  });

  const generate = useCallback((_prompt: string) => {
    setState((s) => ({ ...s, generating: true, tokens: [], error: null }));
    setState((s) => ({
      ...s,
      generating: false,
      error: "Inference engine not connected (Phase 0 placeholder)",
    }));
  }, []);

  const reset = useCallback(() => {
    setState({ generating: false, tokens: [], tokensPerSec: 0, error: null });
  }, []);

  return { ...state, generate, reset };
}
