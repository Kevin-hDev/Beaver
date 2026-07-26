import { Component, type ErrorInfo, type ReactNode } from "react";
import i18n from "@/i18n";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(_error: Error, _info: ErrorInfo) {
    // The visible crash message is intentionally generic.
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          padding: "var(--space-2xl)",
          color: "var(--signal-error)",
          textAlign: "center",
          fontSize: "var(--text-sm)",
        }}>
          {i18n.t("errors.crashMessage")}
        </div>
      );
    }
    return this.props.children;
  }
}
