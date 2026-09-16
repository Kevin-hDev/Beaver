export interface ExtensionHostActivity {
  events: {
    queued: number;
    delivered: number;
    dropped: number;
    timedOut: number;
    activeHandlers: number;
  };
  activeInterceptors: number;
}
