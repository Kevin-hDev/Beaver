import { useEffect } from "react";
import { IS_MAC } from "@/lib/platform";

export function usePlatformBodyClass() {
  useEffect(() => {
    const platformClass = IS_MAC ? "os-mac" : "os-other";
    const oppositeClass = IS_MAC ? "os-other" : "os-mac";

    document.body.classList.remove(oppositeClass);
    document.body.classList.add(platformClass);

    return () => {
      document.body.classList.remove(platformClass);
    };
  }, []);
}
