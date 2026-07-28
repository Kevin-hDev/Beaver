import { Component, type ErrorInfo, type ReactNode } from "react";
import i18n from "@/i18n";
import "./extensions-error-boundary.css";

interface ExtensionsErrorBoundaryProps {
  children: ReactNode;
  resetKey: string;
  onReset: () => void;
}

interface ExtensionsErrorBoundaryState {
  hasError: boolean;
}

export class ExtensionsErrorBoundary extends Component<
  ExtensionsErrorBoundaryProps,
  ExtensionsErrorBoundaryState
> {
  state: ExtensionsErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(): ExtensionsErrorBoundaryState {
    return { hasError: true };
  }

  componentDidCatch(_error: Error, _info: ErrorInfo) {
    // Ne pas journaliser les données ou chemins potentiellement fournis par l'extension.
  }

  componentDidUpdate(previous: ExtensionsErrorBoundaryProps) {
    if (this.state.hasError && previous.resetKey !== this.props.resetKey) {
      this.setState({ hasError: false });
    }
  }

  private reset = () => {
    this.props.onReset();
    this.setState({ hasError: false });
  };

  render() {
    if (!this.state.hasError) return this.props.children;
    return (
      <div className="exeb-fallback" role="alert">
        <p>{i18n.t("extensions.errors.view")}</p>
        <button type="button" className="wk-btn-secondary" onClick={this.reset}>
          {i18n.t("extensions.actions.back")}
        </button>
      </div>
    );
  }
}
