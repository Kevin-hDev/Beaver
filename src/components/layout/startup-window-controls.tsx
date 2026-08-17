import { WindowControls } from "./window-controls";
import "./startup-window-controls.css";

/* Boutons de fenêtre des écrans d'avant-application : splash, accueil,
   installation d'Ollama. Sur Linux et Windows, le démarrage retire les
   décorations natives (`set_decorations(false)`) et l'application dessine ses
   propres boutons — mais ils ne vivaient que dans la coquille de l'application.
   Avant d'y arriver, la fenêtre ne pouvait être ni fermée, ni réduite, ni
   agrandie.
   Le conteneur n'existe que pour passer devant le calque du splash ; les
   boutons eux-mêmes restent la primitive unique de l'application, sans quoi
   leur position finirait par diverger d'un écran à l'autre. */
export function StartupWindowControls() {
  return (
    <div className="startup-window-controls">
      <WindowControls />
    </div>
  );
}
