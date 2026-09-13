import beaverSvg from "../../docs/fonctionnalites/install-&-update/assets/QPXAq01-anime.svg?raw";

interface Props {
  size: "large" | "small";
  frozen?: boolean;
  muted?: boolean;
}

export function InstallerBeaver({ size, frozen = false, muted = false }: Props) {
  const classes = [
    "binst-beaver",
    `binst-beaver-${size}`,
    frozen && "binst-beaver-frozen",
    muted && "binst-beaver-muted",
  ]
    .filter(Boolean)
    .join(" ");
  return <span className={classes} aria-hidden="true" dangerouslySetInnerHTML={{ __html: beaverSvg }} />;
}
