const repeated = (text) => Array.from({ length: 20 }, () => text).join(" ");

export default function activate(beaver) {
  beaver.ui.register({
    type: "tab", id: "multilingual", placement: "app.navigation.primary", order: 60,
    label: {
      default: "Multilingual acceptance",
      fr: repeated("Éléments français accessibles et vérifiés"),
      de: repeated("Deutsche Oberflächenelemente werden zuverlässig geprüft"),
      zh: repeated("中文界面元素经过完整验证"),
      ja: repeated("日本語のインターフェース要素を完全に検証します"),
    },
    detail: { type: "text", text: { default: "Unicode acceptance" } },
  });
}
