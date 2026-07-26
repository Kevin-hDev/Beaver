import { useCallback, useEffect, useRef } from "react";

export function useLatestRequest() {
  const generationRef = useRef(0);

  useEffect(() => () => {
    generationRef.current += 1;
  }, []);

  return useCallback(async <T>(request: () => Promise<T>): Promise<T | undefined> => {
    const generation = ++generationRef.current;
    try {
      const result = await request();
      return generation === generationRef.current ? result : undefined;
    } catch (error) {
      if (generation === generationRef.current) throw error;
      return undefined;
    }
  }, []);
}
