import { useTranslation } from "react-i18next";
import { BeaverWordmark } from "@/components/ui/beaver-wordmark";
import "./welcome-wordmark.css";

interface WelcomeWordmarkProps {
  leaving: boolean;
}

export function WelcomeWordmark({ leaving }: WelcomeWordmarkProps) {
  const { t } = useTranslation();

  return (
    <div className={`wm-wrap ${leaving ? "wm-leaving" : ""}`}>
      {/* Deux éléments et non un seul : l'apparition anime un flou, et le creux du
          logotype est lui aussi un filtre. Portés par le même élément, l'animation
          écraserait le creux. La classe wm-title est le point d'accroche de la
          sortie — welcome-leave.ts attend la fin de l'animation portée ici. */}
      <div className="wm-title">
        <BeaverWordmark title={t("welcome.title")} />
      </div>
    </div>
  );
}
