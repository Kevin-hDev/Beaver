# Régénère l'icône macOS de l'application.
#   uv run --with pillow python3 src-tauri/scripts/build_macos_icon.py
#
# macOS n'ajoute aucune marge de lui-même : il affiche l'image telle quelle.
# Une image qui remplit toute sa toile paraît donc un quart plus large que
# toutes ses voisines du Dock, et c'était le cas de Beaver avant ce script.

import os
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageFilter

# Le maître est l'image de marque approuvée, pleine toile, dont l'empreinte est
# verrouillée par src/components/ui/__tests__/brand-assets.test.ts. Ce script la
# lit sans jamais l'écrire : deux exécutions produisent donc le même fichier,
# alors que relire l'icône déjà générée la rognerait un peu plus à chaque fois.
MASTER = "src/assets/logo.png"

# La silhouette du système. Son côté fixe la taille du dessin dans la toile, et
# sa courbe fixe l'arrondi : c'est la seule autorité pour les deux.
#
# Elle est relevée sur les icônes de macOS 27 — Notes, Calculatrice, Rappels et
# Pense-bêtes donnent exactement la même au pixel près. Un fichier de forme
# plutôt qu'une formule parce que le coin d'Apple n'en a pas de simple : ni un
# arc de cercle ni une superellipse ne le suivent, les deux s'en écartent de
# quatre pixels sur une toile de 256. La silhouette rééchantillonnée, elle,
# retombe à un pixel de désaccord sur 65 536.
SHAPE = "src-tauri/icons/macos-shape.png"
CANVAS = 1024

# Ombre portée du système, ajustée sur son profil d'opacité relevé tous les
# quatre pixels le long des quatre bords. L'écart moyen restant vaut un demi
# point d'alpha sur 255.
SHADOW_ALPHA = 48
SHADOW_OFFSET = 6
SHADOW_BLUR = 16

# Les dix noms qu'iconutil attend, avec le côté en pixels de chacun.
TILES = {
    "icon_16x16.png": 16,
    "icon_16x16@2x.png": 32,
    "icon_32x32.png": 32,
    "icon_32x32@2x.png": 64,
    "icon_128x128.png": 128,
    "icon_128x128@2x.png": 256,
    "icon_256x256.png": 256,
    "icon_256x256@2x.png": 512,
    "icon_512x512.png": 512,
    "icon_512x512@2x.png": 1024,
}

OUTPUT = "src-tauri/icons/icon.icns"
ERROR_MESSAGE = "macOS icon build failed"


# Le préfixe est posé une seule fois, à l'affichage : le porter aussi ici le
# ferait apparaître deux fois dans la ligne d'erreur.
class IconBuildError(Exception):
    pass


def load_shape(root: Path) -> Image.Image:
    path = root / SHAPE
    if not path.is_file():
        raise IconBuildError(f"missing shape {SHAPE}")
    shape = Image.open(path).convert("L")
    side, height = shape.size
    if side != height:
        raise IconBuildError(f"shape is {shape.size}, expected a square")
    if not CANVAS * 3 // 4 < side < CANVAS:
        raise IconBuildError(f"shape side {side} leaves no margin in {CANVAS}")
    return shape


def load_master(root: Path) -> Image.Image:
    path = root / MASTER
    if not path.is_file():
        raise IconBuildError(f"missing master {MASTER}")
    master = Image.open(path).convert("RGBA")
    if master.size != (CANVAS, CANVAS):
        raise IconBuildError(f"master is {master.size}, expected {CANVAS} square")
    return master


def build_master_tile(master: Image.Image, shape: Image.Image) -> Image.Image:
    body_side = shape.size[0]
    margin = (CANVAS - body_side) // 2

    body = master.resize((body_side, body_side), Image.LANCZOS)
    body.putalpha(shape)

    spread = Image.new("L", (CANVAS, CANVAS), 0)
    spread.paste(shape, (margin, margin + SHADOW_OFFSET))
    spread = spread.filter(ImageFilter.GaussianBlur(SHADOW_BLUR))
    shadow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    shadow.putalpha(spread.point(lambda value: value * SHADOW_ALPHA // 255))

    tile = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    tile.alpha_composite(shadow)
    tile.alpha_composite(body, (margin, margin))
    return tile


def check_geometry(tile: Image.Image, body_side: int) -> None:
    margin = (CANVAS - body_side) // 2
    alpha = tile.getchannel("A").point(lambda value: 255 if value >= 128 else 0)
    box = alpha.getbbox()
    if box != (margin, margin, CANVAS - margin, CANVAS - margin):
        raise IconBuildError(f"unexpected body box {box}")
    pixels = alpha.load()
    # Le coin de la boîte doit être creusé et le milieu du bord haut plein :
    # sans les deux, un arrondi absent ou débordant passerait inaperçu.
    if pixels[margin + 4, margin + 4] != 0:
        raise IconBuildError("corner is not rounded")
    if pixels[CANVAS // 2, margin + 2] != 255:
        raise IconBuildError("top edge is carved")


def write_icns(tile: Image.Image, destination: Path) -> None:
    with tempfile.TemporaryDirectory() as workspace:
        iconset = Path(workspace) / "icon.iconset"
        iconset.mkdir()
        for name, size in TILES.items():
            tile.resize((size, size), Image.LANCZOS).save(iconset / name)
        built = Path(workspace) / "icon.icns"
        subprocess.run(
            ["iconutil", "--convert", "icns", str(iconset), "--output", str(built)],
            check=True,
            capture_output=True,
        )
        # Remplacement atomique : une écriture interrompue laisserait sinon une
        # icône tronquée, que le build embarquerait sans rien signaler.
        staged = destination.with_name(destination.name + ".tmp")
        staged.write_bytes(built.read_bytes())
        os.replace(staged, destination)


def main() -> int:
    if sys.platform != "darwin":
        print(f"{ERROR_MESSAGE}: iconutil only exists on macOS", file=sys.stderr)
        return 1
    root = Path(__file__).resolve().parents[2]
    try:
        shape = load_shape(root)
        tile = build_master_tile(load_master(root), shape)
        check_geometry(tile, shape.size[0])
        write_icns(tile, root / OUTPUT)
    except (IconBuildError, subprocess.CalledProcessError, OSError) as error:
        print(f"{ERROR_MESSAGE}: {error}", file=sys.stderr)
        return 1
    side = shape.size[0]
    print(f"wrote {OUTPUT} — body {side}px, margin {(CANVAS - side) // 2}px")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
