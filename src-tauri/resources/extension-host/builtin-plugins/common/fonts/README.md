# Bundled PDF fonts

Beaver bundles fixed Noto font files so official PDF creation works offline.
Only the fonts required by a document are loaded, verified, cached, and
embedded.

Coverage includes:

- Latin, Greek, Cyrillic, CJK, Arabic, arrows, symbols, and dingbats;
- Armenian, Hebrew, Devanagari, Bengali, Tamil, Sinhala, Thai, Lao, Tibetan,
  Myanmar, Georgian, Ethiopic, Cherokee, and Khmer;
- monochrome vector emoji supported by `NotoEmoji-Regular.ttf`.

The exact filename, byte count, SHA-256 digest, and pinned upstream commit for
every asset live in `catalog.mjs`. Runtime loading fails closed when a file
does not match that catalog. The main Noto distribution is pinned to
`notofonts/notofonts.github.io@eaa1a5cf8cb83ea73941197e492d659e51bb11dd`;
CJK and Arabic use their separate repositories, while Noto Emoji comes from
the official Google Fonts distribution. Their exact commits are recorded in
the catalog.

All bundled fonts are distributed under the SIL Open Font License 1.1 in
`OFL.txt`.
