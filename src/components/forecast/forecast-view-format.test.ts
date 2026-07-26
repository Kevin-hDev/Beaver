import { describe, it, expect } from "vitest";
import {
  inferMetricMeta,
  formatForecastValue,
  buildPeriodMeta,
  resolvePredictionDate,
} from "./forecast-view-format";

// --- inferMetricMeta --------------------------------------------------------

describe("inferMetricMeta", () => {
  it("détecte une métrique monétaire (EUR) en français", () => {
    const meta = inferMetricMeta("fr-FR", "revenue");
    expect(meta.unitKind).toBe("currency-eur");
    expect(meta.unitLabel).toBe("€");
  });

  it("détecte une métrique monétaire (EUR) en anglais", () => {
    const meta = inferMetricMeta("en", "sales_eur");
    expect(meta.unitKind).toBe("currency-eur");
    expect(meta.unitLabel).toBe("EUR");
  });

  it("détecte une métrique de comptage (units)", () => {
    const meta = inferMetricMeta("fr-FR", "qty");
    expect(meta.unitKind).toBe("count");
    expect(meta.unitLabel).toBe("unités");
  });

  it("détecte les commandes comme comptage", () => {
    const meta = inferMetricMeta("en", "nb_orders");
    expect(meta.unitKind).toBe("count");
  });

  it("retourne generic pour une colonne inconnue", () => {
    const meta = inferMetricMeta("fr-FR", "temperature");
    expect(meta.unitKind).toBe("generic");
    expect(meta.unitLabel).toBeNull();
  });

  it("utilise le fallbackName si targetColumn absent", () => {
    const meta = inferMetricMeta("fr-FR", undefined, "Forecast revenue 2026");
    expect(meta.unitKind).toBe("currency-eur");
  });

  it("prettify le nom de colonne", () => {
    const meta = inferMetricMeta("fr-FR", "sales_eur");
    expect(meta.columnTitle).toBe("Sales Eur");
  });

  it("retourne 'forecast' si aucune info", () => {
    const meta = inferMetricMeta("fr-FR", undefined, undefined);
    expect(meta.columnTitle.toLowerCase()).toBe("forecast");
  });
});

// --- formatForecastValue ----------------------------------------------------

describe("formatForecastValue", () => {
  const currencyMeta = inferMetricMeta("fr-FR", "revenue");
  const genericMeta = inferMetricMeta("fr-FR", "temperature");

  it("formate une valeur monétaire en EUR", () => {
    const result = formatForecastValue(1234.5, "fr-FR", currencyMeta);
    // Format dépendant de la locale, mais doit contenir 2 décimales et €/EUR.
    expect(result).toMatch(/1[\s\u202f]?234,50/);
    expect(result).toMatch(/€/);
  });

  it("formate une valeur générique sans devise", () => {
    const result = formatForecastValue(42.5, "fr-FR", genericMeta);
    expect(result).toContain("42,5");
    expect(result).not.toContain("€");
  });

  it("gère zéro", () => {
    const result = formatForecastValue(0, "fr-FR", genericMeta);
    expect(result).toContain("0");
  });

  it("gère les valeurs négatives", () => {
    const result = formatForecastValue(-100, "fr-FR", currencyMeta);
    expect(result).toContain("100");
  });
});

// --- resolvePredictionDate --------------------------------------------------

describe("resolvePredictionDate", () => {
  it("retourne une date explicite si elle est parsable", () => {
    const result = resolvePredictionDate("2026-01-15", "2026-01-10", "D", 4);
    expect(result).not.toBeNull();
    expect(result?.getDate()).toBe(15);
  });

  it("retourne une date pour une ISO datetime complète", () => {
    const result = resolvePredictionDate("2026-06-15T12:00:00", "2026-01-10", "D", 0);
    expect(result).not.toBeNull();
    expect(result?.getHours()).toBe(12);
  });

  it("retourne null pour une chaîne garbage non-parsable", () => {
    // "not-a-date" n'est parsé par ni new Date, ni le regex T+\d+.
    expect(resolvePredictionDate("not-a-date", "2026-01-10", "D", 0)).toBeNull();
  });

  it("retourne null pour une chaîne vide", () => {
    expect(resolvePredictionDate("", "2026-01-10", "D", 0)).toBeNull();
  });

  it("parse un placeholder T+N comme date explicite (comportement V8 permissif)", () => {
    // Note : new Date('T+5') réussit à parser en V8 → la branche shift n'est
    // pas atteinte. Ce test documente ce comportement.
    const result = resolvePredictionDate("T+5", "2026-01-10", "D", 4);
    expect(result).not.toBeNull();
  });
});

// --- buildPeriodMeta --------------------------------------------------------

describe("buildPeriodMeta", () => {
  it("construit un stepLabel T+index", () => {
    const meta = buildPeriodMeta(0, "2026-01-10", "2026-01-10", "D", "fr-FR");
    expect(meta.stepLabel).toBe("T+1");
  });

  it("construit un stepLabel T+2 pour index 1", () => {
    const meta = buildPeriodMeta(1, "2026-01-11", "2026-01-10", "D", "fr-FR");
    expect(meta.stepLabel).toBe("T+2");
  });

  it("retourne un secondaryLabel formaté avec la date résolue", () => {
    const meta = buildPeriodMeta(0, "2026-01-10", "2026-01-10", "D", "fr-FR");
    // La date doit apparaître dans le label secondaire.
    expect(meta.secondaryLabel).toContain("10");
  });

  it("retourne rawDate comme secondaryLabel si non résolvable", () => {
    const meta = buildPeriodMeta(0, "garbage", "2026-01-10", "D", "fr-FR");
    expect(meta.secondaryLabel).toBe("garbage");
  });

  it("masque l'heure pour une fréquence mensuelle (artefact de fuseau)", () => {
    const meta = buildPeriodMeta(0, "2026-07-01T02:00:00", "2026-06-01", "M", "fr-FR");
    expect(meta.secondaryLabel).not.toContain("02:00");
    expect(meta.secondaryLabel).toContain("01/07/2026");
  });

  it("conserve l'heure pour une fréquence journalière", () => {
    const meta = buildPeriodMeta(0, "2026-07-01T02:00:00", "2026-06-30", "D", "fr-FR");
    expect(meta.secondaryLabel).toContain("02:00");
  });
});
