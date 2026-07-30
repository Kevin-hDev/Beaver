import beaverSheet from "@/assets/mascot/cl-go-beaver/master.webp";
import beaverManifest from "@/assets/mascot/cl-go-beaver/manifest.json";
import circuitActionsSheet from "@/assets/mascot/circuit/actions.webp";
import circuitManifest from "@/assets/mascot/circuit/manifest.json";
import circuitStandardSheet from "@/assets/mascot/circuit/standard.webp";
import kovaActionsSheet from "@/assets/mascot/kova/actions.webp";
import kovaManifest from "@/assets/mascot/kova/manifest.json";
import kovaStandardSheet from "@/assets/mascot/kova/standard.webp";
import mokaiActionsSheet from "@/assets/mascot/mokai/actions.webp";
import mokaiManifest from "@/assets/mascot/mokai/manifest.json";
import mokaiStandardSheet from "@/assets/mascot/mokai/standard.webp";
import nivalActionsSheet from "@/assets/mascot/nival/actions.webp";
import nivalManifest from "@/assets/mascot/nival/manifest.json";
import nivalStandardSheet from "@/assets/mascot/nival/standard.webp";
import picoActionsSheet from "@/assets/mascot/pico/actions.webp";
import picoManifest from "@/assets/mascot/pico/manifest.json";
import picoStandardSheet from "@/assets/mascot/pico/standard.webp";
import rakuActionsSheet from "@/assets/mascot/raku/actions.webp";
import rakuManifest from "@/assets/mascot/raku/manifest.json";
import rakuStandardSheet from "@/assets/mascot/raku/standard.webp";
import voltActionsSheet from "@/assets/mascot/volt/actions.webp";
import voltManifest from "@/assets/mascot/volt/manifest.json";
import voltStandardSheet from "@/assets/mascot/volt/standard.webp";
import type { MascotId } from "@/types/mascot";
import type { MascotBundle, MascotManifest } from "./mascot-bundle-types";

function createTwoSheetBundle(
  manifest: MascotManifest & {
    sheets: {
      standard: { columns: number; rows: number };
      actions: { columns: number; rows: number };
    };
  },
  standardSheet: string,
  actionsSheet: string,
): MascotBundle {
  return {
    manifest,
    defaultSheet: "standard",
    sheets: {
      standard: { src: standardSheet, ...manifest.sheets.standard },
      actions: { src: actionsSheet, ...manifest.sheets.actions },
    },
  };
}

export const MASCOT_BUNDLES: Record<MascotId, MascotBundle> = {
  "cl-go-beaver": {
    manifest: beaverManifest,
    defaultSheet: "master",
    sheets: {
      master: {
        src: beaverSheet,
        columns: beaverManifest.columns,
        rows: beaverManifest.states.length,
      },
    },
  },
  circuit: createTwoSheetBundle(
    circuitManifest,
    circuitStandardSheet,
    circuitActionsSheet,
  ),
  kova: createTwoSheetBundle(kovaManifest, kovaStandardSheet, kovaActionsSheet),
  nival: createTwoSheetBundle(
    nivalManifest,
    nivalStandardSheet,
    nivalActionsSheet,
  ),
  mokai: createTwoSheetBundle(mokaiManifest, mokaiStandardSheet, mokaiActionsSheet),
  volt: createTwoSheetBundle(voltManifest, voltStandardSheet, voltActionsSheet),
  raku: createTwoSheetBundle(rakuManifest, rakuStandardSheet, rakuActionsSheet),
  pico: createTwoSheetBundle(picoManifest, picoStandardSheet, picoActionsSheet),
};
