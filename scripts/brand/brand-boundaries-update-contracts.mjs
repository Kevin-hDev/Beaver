const contract = (name, file, snippets) => ({ name, file, snippets });

export const UPDATE_COMPATIBILITY_CONTRACTS = Object.freeze([
  contract("protocole du helper de mise à jour", "src-tauri/src/updater_worker/mod.rs", [
    '"cl-go-dash-update"',
  ]),
  contract("argument de santé de la mise à jour", "src-tauri/src/services/update_health.rs", [
    '"--clgo-update-health"',
  ]),
  contract("bundle historique de la mise à jour macOS", "src-tauri/src/updater_worker/macos_bundle.rs", [
    'Some("CL-GO.app")',
    'const BUNDLE_IDENTIFIER: &str = "com.clgo.dash";',
    'const EXECUTABLE_NAME: &str = "cl-go-dash";',
  ]),
  contract("structure historique de l’application macOS", "src-tauri/src/updater_worker/verify.rs", [
    'Some("CL-GO.app" | "Beaver.app")',
  ]),
  contract("préparation du helper de mise à jour", "scripts/build/prepare-updater-helper.mjs", [
    "createUpdaterBuildPlan",
  ]),
  contract("métadonnées de la version-pont historique", "scripts/release/check-bridge-metadata.mjs", [
    'const INTERNAL_NAME = "cl-go-dash";',
    'const PRODUCT_NAME = "CL-GO";',
    'const IDENTIFIER = "com.clgo.dash";',
  ]),
  contract("publication de la version-pont historique", "scripts/release/publish-bridge-release.mjs", [
    'const HISTORICAL_REPOSITORY = "Kevin-hDev/CL-GO-DASH";',
    "`CL-GO_${version}_aarch64.dmg`",
    "`CL-GO_${version}_amd64.deb`",
    "`CL-GO_${version}_x64-setup.exe`",
    "value.name !== `CL-GO ${tagValue}`",
  ]),
]);
