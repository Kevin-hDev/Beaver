import { selectedArtifact, selectedBuildTool } from "./cef-artifacts.mjs";
import { verifiedArchive } from "./cef-download.mjs";
import { extractVerifiedArchive } from "./cef-extract.mjs";
import { prepareBuildTool } from "./cef-tool.mjs";

async function main() {
  const artifact = selectedArtifact();
  if (artifact === null) return;
  const tool = selectedBuildTool();
  await prepareBuildTool(tool);
  const archive = await verifiedArchive(artifact);
  await extractVerifiedArchive(archive, artifact);
  process.stdout.write("CEF source verified with SHA-256\n");
}

main().catch(() => {
  process.stderr.write("CEF source preparation failed\n");
  process.exitCode = 1;
});
