import { afterEach, describe, expect, it, vi } from "vitest";
import {
  acquireBrowserNativeOcclusion,
  BROWSER_NATIVE_OCCLUSION_EVENT,
} from "../browser-native-occlusion";

describe("browser native occlusion", () => {
  afterEach(() => vi.restoreAllMocks());

  it("attend l'image suivante avant de publier la révélation", () => {
    const frames: FrameRequestCallback[] = [];
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((callback) => {
      frames.push(callback);
      return frames.length;
    });
    let notifications = 0;
    const countNotification = () => { notifications += 1; };
    window.addEventListener(BROWSER_NATIVE_OCCLUSION_EVENT, countNotification);

    const release = acquireBrowserNativeOcclusion();
    expect(release).not.toBeNull();
    notifications = 0;
    release?.();

    expect(notifications).toBe(0);
    expect(frames).toHaveLength(1);
    frames[0](1);
    expect(notifications).toBe(1);
    window.removeEventListener(BROWSER_NATIVE_OCCLUSION_EVENT, countNotification);
  });
});
