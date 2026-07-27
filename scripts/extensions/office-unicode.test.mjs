import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  callOffice,
  createOfficeHost,
  fixtureDirectory,
  syncOfficePlugins,
} from "./office-test-helpers.mjs";

const SAMPLES = [
  "日本語",
  "中文",
  "한국어",
  "Русский",
  "Symboles • — « » → € ✓",
  "العربية",
  "Բարեւ աշխարհ",
  "שלום עולם",
  "नमस्ते दुनिया",
  "নমস্কার বিশ্ব",
  "வணக்கம் உலகம்",
  "හෙලෝ ලෝකය",
  "สวัสดีชาวโลก",
  "ສະບາຍດີໂລກ",
  "བཀྲ་ཤིས་བདེ་ལེགས",
  "မင်္ဂလာပါ ကမ္ဘာ",
  "გამარჯობა მსოფლიო",
  "ሰላም ዓለም",
  "ᎣᏏᏲ ᎡᎶᎯ",
  "សួស្តីពិភពលោក",
  "Bilan 📊 validé ✅ statut 🟢 lancement 🚀",
];

test("creates an offline Unicode PDF across Beaver's supported scripts", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-unicode-pdf-"));
  const host = createOfficeHost();
  try {
    await syncOfficePlugins(host);
    const externalText = await readFile(
      join(fixtureDirectory, "external-unicode-samples.txt"),
      "utf8",
    );
    const created = await callOffice(host, workspace, "beaver.office.pdf.create", {
      path: "unicode.pdf",
      title: externalText,
      paragraphs: [...SAMPLES, "界".repeat(500)],
    });
    assert.notEqual(created.isError, true);
    assert.equal((await readFile(join(workspace, "unicode.pdf"))).subarray(0, 5).toString(), "%PDF-");

    const inspected = await callOffice(host, workspace, "beaver.office.pdf.inspect", {
      path: "unicode.pdf",
      maxPages: 20,
    });
    assert.notEqual(inspected.isError, true);
    const extracted = JSON.parse(inspected.content).pages
      .map((page) => page.text)
      .join(" ");
    for (const sample of SAMPLES) {
      assert.equal(extracted.includes(sample), true, sample);
    }
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});

test("rejects an uncovered script without losing the extension host", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-unsupported-pdf-"));
  const host = createOfficeHost();
  try {
    await syncOfficePlugins(host);
    const result = await callOffice(host, workspace, "beaver.office.pdf.create", {
      path: "unsupported.pdf",
      paragraphs: ["Nag Mundari: \u{1E4D0}"],
    });

    assert.equal(result.isError, true);
    assert.equal(result.content, "unsupported_character");
    assert.equal((await host.request("host.hello", {})).apiVersion, "1");
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});

test("a Latin PDF does not embed the large CJK font", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-lazy-font-pdf-"));
  const host = createOfficeHost();
  try {
    await syncOfficePlugins(host);
    await callOffice(host, workspace, "beaver.office.pdf.create", {
      path: "latin.pdf",
      paragraphs: ["Bonjour, Beaver crée ce document sans charger le CJK."],
    });
    const pdf = await readFile(join(workspace, "latin.pdf"));
    assert.ok(pdf.length < 100_000);
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});
