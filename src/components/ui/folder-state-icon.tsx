import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

interface FolderStateIconProps extends InlineIconProps {
  open: boolean;
}

/* Deux dessins et non un dessin pivoté : le dossier ouvert laisse voir son
   intérieur, le fermé non. C'est ce que la ligne annonce — le projet est
   déplié, ou il ne l'est pas — et une flèche qui tourne ne le dit pas. */
const OPEN_PATH = "M20.361 18.58c-.405.39-.943.641-1.536.684l-1.638.117a73 73 0 0 1-10.374 0l-1.514-.108a2.63 2.63 0 0 1-2.398-2.15a24.2 24.2 0 0 1-.222-7.244L2.95 7.61a2.68 2.68 0 0 1 2.66-2.36h2.292c1.118 0 2.05.798 2.255 1.856h8.314c1.307 0 2.42.95 2.625 2.24l.064.4l.04.254h.335a2.093 2.093 0 0 1 1.951 2.852l-1.25 3.213a5.9 5.9 0 0 1-1.876 2.514m-.745-8.998l.064.401q0 .008.003.017H10.37a2.75 2.75 0 0 0-2.565 1.757L5.473 17.78l-.068-.005a1.13 1.13 0 0 1-1.03-.922a22.7 22.7 0 0 1-.208-6.796l.273-2.27A1.18 1.18 0 0 1 5.61 6.75h2.292c.44 0 .797.357.797.797c0 .585.474 1.06 1.06 1.06h8.712c.57 0 1.054.413 1.144.975M7.039 17.893a71 71 0 0 0 10.041-.008l1.638-.118l.195-.018l-.002-.002a4.38 4.38 0 0 0 1.929-2.226l1.25-3.213a.593.593 0 0 0-.554-.808H10.37c-.516 0-.979.317-1.165.799z";
const CLOSED_PATH = "M19.602 16.976c.422-2.31.448-4.674.078-6.993l-.064-.401a1.16 1.16 0 0 0-1.144-.976H9.76a1.06 1.06 0 0 1-1.06-1.06a.797.797 0 0 0-.797-.796H5.612c-.597 0-1.1.446-1.171 1.039l-.273 2.269a22.7 22.7 0 0 0 .208 6.796c.093.506.516.886 1.03.922l1.514.109c3.382.242 6.778.242 10.16 0l1.638-.118a.97.97 0 0 0 .884-.791m1.56-7.23a22.2 22.2 0 0 1-.085 7.5a2.47 2.47 0 0 1-2.252 2.018l-1.638.117a73 73 0 0 1-10.374 0l-1.514-.108a2.63 2.63 0 0 1-2.398-2.15a24.2 24.2 0 0 1-.222-7.244L2.95 7.61a2.68 2.68 0 0 1 2.66-2.36h2.292c1.118 0 2.05.798 2.255 1.856h8.314c1.307 0 2.42.95 2.625 2.24z";

export function FolderStateIcon({ open, size = "var(--project-folder-icon-size)", className }: FolderStateIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path fill="currentColor" fillRule="evenodd" clipRule="evenodd" d={open ? OPEN_PATH : CLOSED_PATH} />
    </InlineIcon>
  );
}
