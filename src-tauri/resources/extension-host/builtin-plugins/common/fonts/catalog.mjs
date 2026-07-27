const NOTO_DISTRIBUTION = "notofonts/notofonts.github.io@eaa1a5cf8cb83ea73941197e492d659e51bb11dd";

export const PDF_FONT_SPECS = Object.freeze({
  base: spec("NotoSans-Regular.ttf", 431_364, "f3961a9cde016d41a4879aecda1474d3a36d6bf54fa0e4643de029cc2248b0e8", NOTO_DISTRIBUTION),
  symbols: spec("NotoSansSymbols-Regular.ttf", 264_032, "0088617baec0e8ac47e022cc1f38695f772301c9ef6d1f24a785abbef1e05d79", NOTO_DISTRIBUTION),
  symbols2: spec("NotoSansSymbols2-Regular.ttf", 1_233_128, "3ce38effdb615dd929c8b0f52768dfa2cd21f206e3824bc7df61e4074b41ae52", NOTO_DISTRIBUTION),
  cjk: spec("NotoSansCJKjp-Regular.otf", 16_467_736, "68a3fc98800b2a27b371f2fb79991daf3633bd89309d4ffaa6946fd587f375b5", "notofonts/noto-cjk@f8d157532fbfaeda587e826d4cd5b21a49186f7c"),
  arabic: spec("NotoSansArabic-Regular.ttf", 240_456, "ceea25b464a656dc3b26849bab9356740401af62aedf1bfa8b7f0d9b75925b1b", "notofonts/noto-fonts@ffebf8c1ee449e544955a7e813c54f9b73848eac"),
  armenian: spec("NotoSansArmenian-Regular.ttf", 18_772, "73bde9f3c63aa5ed39236e8a2837224605210a98b54d65724676c0119fcaa24c", NOTO_DISTRIBUTION),
  hebrew: spec("NotoSansHebrew-Regular.ttf", 16_836, "04272f5600d0ec816d31d0df73b23aa8d3501ea359ebe820da31c11ffcf00853", NOTO_DISTRIBUTION),
  devanagari: spec("NotoSansDevanagari-Regular.ttf", 185_012, "216921eded5a97435fa0638deca66496bf51f52fa3467f566deb9938c25a71de", NOTO_DISTRIBUTION),
  bengali: spec("NotoSansBengali-Regular.ttf", 103_704, "5dceda02816fece18ea6796f474f5d3170f1d862f21b1e809cdc94b1caa4b6ec", NOTO_DISTRIBUTION),
  tamil: spec("NotoSansTamil-Regular.ttf", 46_600, "6634d9cc97a726e670df41281dd32b167a6b9b71f2036e19671ff08fdde0c292", NOTO_DISTRIBUTION),
  sinhala: spec("NotoSansSinhala-Regular.ttf", 90_296, "46d5b54952a624e6e3981e0968aecc5464f3b1081131dd7a1fcd42fbf7966471", NOTO_DISTRIBUTION),
  thai: spec("NotoSansThai-Regular.ttf", 20_960, "d4303fe9c63ebb72759ca8b6d2040c8ae81689f7d08d7b91c656154382b49313", NOTO_DISTRIBUTION),
  lao: spec("NotoSansLao-Regular.ttf", 21_720, "d3976b0ac08702c54999ff42eae295d1597fd73b44e0c57d7f4917524e18afbe", NOTO_DISTRIBUTION),
  tibetan: spec("NotoSerifTibetan-Regular.ttf", 609_640, "7292b2c76cf5c9b81e5362d88e0cb0a16b9d92827f86f945203014b049fd2fc5", NOTO_DISTRIBUTION),
  myanmar: spec("NotoSansMyanmar-Regular.ttf", 148_628, "f4be6ef43871516d8347c5217a2033991b8ecda2dd1200007c2fb431a66d64db", NOTO_DISTRIBUTION),
  georgian: spec("NotoSansGeorgian-Regular.ttf", 35_300, "f4b229b126859725b75031dd8c051335d20c85e4eb6d525af28f3e83e471baa4", NOTO_DISTRIBUTION),
  ethiopic: spec("NotoSansEthiopic-Regular.ttf", 289_244, "86b9c07c049e68438388d00a34033fa28cabeef91b26e1bbed67512d360166d4", NOTO_DISTRIBUTION),
  cherokee: spec("NotoSansCherokee-Regular.ttf", 65_168, "4460410f1a089f79af329d5d26e09381ba553a05baa7c1f3c352fc43bb96878d", NOTO_DISTRIBUTION),
  khmer: spec("NotoSansKhmer-Regular.ttf", 67_784, "60c10dbfae33a44f1897cd939789bd173aedb7c7dc0a9af389a737e33cb7e548", NOTO_DISTRIBUTION),
  emoji: spec("NotoEmoji-Regular.ttf", 1_982_596, "de6c18832938afc99caf132b39d6a30a19bac7f2e812e28db2535b4608d27551", "google/fonts@7ff85c87f93ea6cca5f41c69f2e4edcb90240f26"),
});

function spec(file, bytes, sha256, source) {
  return Object.freeze({
    file,
    bytes,
    sha256,
    source,
    url: new URL(file, import.meta.url),
  });
}
