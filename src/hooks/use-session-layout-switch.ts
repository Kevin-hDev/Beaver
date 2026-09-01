import { useLayoutEffect, useState } from "react";

export function useSessionLayoutSwitch(sessionId: string) {
  const [paintedSessionId, setPaintedSessionId] = useState(sessionId);
  const switching = paintedSessionId !== sessionId;

  useLayoutEffect(() => {
    if (!switching) return;
    let secondFrame = 0;
    /* Deux peintures gardent les transitions coupées jusqu'à ce que le
       navigateur ait appliqué directement la géométrie de la nouvelle session. */
    const firstFrame = requestAnimationFrame(() => {
      secondFrame = requestAnimationFrame(() => {
        setPaintedSessionId(sessionId);
      });
    });
    return () => {
      cancelAnimationFrame(firstFrame);
      if (secondFrame) cancelAnimationFrame(secondFrame);
    };
  }, [sessionId, switching]);

  return switching;
}
