import { useCallback, useMemo, useState } from "react";
import type { ReactNode } from "react";
import type { ThemeChoice } from "@/hooks/use-theme";
import { OllamaSetupScreen } from "@/components/ollama/ollama-setup-screen";
import { IS_MAC } from "@/lib/platform";
import { OnboardingWelcome } from "./onboarding-welcome";
import { OnboardingPreferences } from "./onboarding-preferences";
import { OnboardingAgentImport } from "./onboarding-agent-import";
import { OnboardingApi } from "./onboarding-api";
import "./onboarding.css";
import "./onboarding-form.css";

interface OnboardingScreenProps {
  themeChoice: ThemeChoice;
  onThemeChange: (theme: ThemeChoice) => void;
  showOllamaStep: boolean;
  onCompleteOnboarding: () => void | Promise<void>;
  onCompleteOllama: () => void | Promise<void>;
  onSkipOllama: () => void | Promise<void>;
}

interface OnboardingSlide {
  id: string;
  content: ReactNode;
}

export function OnboardingScreen({
  themeChoice,
  onThemeChange,
  showOllamaStep,
  onCompleteOnboarding,
  onCompleteOllama,
  onSkipOllama,
}: OnboardingScreenProps) {
  const [step, setStep] = useState(0);

  const goNext = useCallback(() => {
    setStep((current) => current + 1);
  }, []);

  const goBack = useCallback(() => {
    setStep((current) => Math.max(0, current - 1));
  }, []);

  const finishApiStep = useCallback(async () => {
    if (showOllamaStep) {
      goNext();
      return;
    }
    await onCompleteOnboarding();
  }, [goNext, onCompleteOnboarding, showOllamaStep]);

  const slides = useMemo<OnboardingSlide[]>(() => [
    { id: "welcome", content: <OnboardingWelcome onNext={goNext} /> },
    {
      id: "preferences",
      content: (
        <OnboardingPreferences
          themeChoice={themeChoice}
          onThemeChange={onThemeChange}
          onNext={goNext}
        />
      ),
    },
    {
      id: "agent-import",
      content: <OnboardingAgentImport onNext={goNext} onBack={goBack} />,
    },
    {
      id: "api",
      content: <OnboardingApi onComplete={finishApiStep} onBack={goBack} />,
    },
    ...(showOllamaStep
      ? [
          {
            id: "ollama",
            content: (
              <div className="ob-page ob-ollama-page">
                <OllamaSetupScreen
                  onComplete={onCompleteOllama}
                  onSkip={onSkipOllama}
                />
              </div>
            ),
          },
        ]
      : []),
  ], [
    finishApiStep,
    goBack,
    goNext,
    onCompleteOllama,
    onSkipOllama,
    onThemeChange,
    showOllamaStep,
    themeChoice,
  ]);

  return (
    <div className={`ob-shell ${IS_MAC ? "os-mac" : "os-other"}`}>
      <div className="ob-track" style={{ transform: `translateX(-${step * 100}%)` }}>
        {slides.map((slide) => (
          <section className="ob-slide" key={slide.id}>
            {slide.content}
          </section>
        ))}
      </div>
    </div>
  );
}
