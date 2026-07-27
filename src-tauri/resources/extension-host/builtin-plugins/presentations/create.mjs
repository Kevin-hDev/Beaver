import { PptxGenJS } from "../common/formats/presentation.mjs";
import { OFFICE_EXTENSIONS, OFFICE_LIMITS } from "../common/constants.mjs";
import { rejectOffice, success } from "../common/errors.mjs";
import {
  boundedArray,
  optionalString,
  plainObject,
  requiredString,
} from "../common/validation.mjs";
import { atomicWrite, workspaceOutput } from "../common/workspace.mjs";
import {
  PRESENTATION_FONTS,
  PRESENTATION_LAYOUT,
  PRESENTATION_THEME,
} from "./theme.mjs";

const LANGUAGE_TAG = /^[a-zA-Z]{2,8}(?:-[a-zA-Z0-9]{1,8}){0,3}$/u;

export async function createPresentation(arguments_, context) {
  const path = requiredString(arguments_?.path, OFFICE_LIMITS.maxPathChars);
  const title = optionalString(arguments_?.title, 300);
  const language = optionalString(arguments_?.language, 35);
  if (language && !LANGUAGE_TAG.test(language)) rejectOffice("invalid_input");
  const themeName = arguments_?.theme ?? "light";
  const theme = PRESENTATION_THEME[themeName];
  if (!theme) rejectOffice("invalid_input");
  const slides = boundedArray(arguments_?.slides, OFFICE_LIMITS.maxSlides)
    .map(validateSlide);
  const totalText = slides.reduce(
    (sum, slide) => sum + slide.title.length
      + slide.bullets.reduce((subtotal, bullet) => subtotal + bullet.length, 0)
      + (slide.notes?.length ?? 0),
    0,
  );
  if (totalText > OFFICE_LIMITS.maxTextChars) rejectOffice("invalid_input");
  const output = await workspaceOutput(
    context,
    path,
    OFFICE_EXTENSIONS.presentation,
  );
  const presentation = new PptxGenJS();
  presentation.layout = "LAYOUT_WIDE";
  presentation.author = "Beaver";
  presentation.company = "Beaver";
  presentation.title = title ?? slides[0].title;
  presentation.subject = "Beaver presentation";
  presentation.theme = {
    headFontFace: PRESENTATION_FONTS.heading,
    bodyFontFace: PRESENTATION_FONTS.body,
    ...(language ? { lang: language } : {}),
  };
  for (const definition of slides) addSlide(presentation, definition, theme);
  const bytes = await presentation.write({
    outputType: "nodebuffer",
    compression: true,
  });
  await atomicWrite(output.path, bytes);
  return success({ path, format: "pptx", slides: slides.length });
}

function addSlide(presentation, definition, theme) {
  const slide = presentation.addSlide();
  slide.background = { color: theme.background };
  slide.addText(definition.title, {
    x: PRESENTATION_LAYOUT.titleX,
    y: PRESENTATION_LAYOUT.titleY,
    w: PRESENTATION_LAYOUT.titleW,
    h: PRESENTATION_LAYOUT.titleH,
    fontFace: PRESENTATION_FONTS.heading,
    fontSize: 28,
    bold: true,
    color: theme.title,
    margin: 0,
    breakLine: false,
  });
  slide.addShape(presentation.ShapeType.rect, {
    x: PRESENTATION_LAYOUT.accentX,
    y: PRESENTATION_LAYOUT.accentY,
    w: PRESENTATION_LAYOUT.accentW,
    h: PRESENTATION_LAYOUT.accentH,
    line: { color: theme.accent, transparency: 100 },
    fill: { color: theme.accent },
  });
  if (definition.bullets.length > 0) {
    const fontSize = definition.bullets.length <= 8
      ? 20
      : definition.bullets.length <= 14 ? 16 : 13;
    const runs = definition.bullets.map((text) => ({
      text,
      options: { bullet: { indent: fontSize }, breakLine: true },
    }));
    slide.addText(runs, {
      x: PRESENTATION_LAYOUT.bodyX,
      y: PRESENTATION_LAYOUT.bodyY,
      w: PRESENTATION_LAYOUT.bodyW,
      h: PRESENTATION_LAYOUT.bodyH,
      fontFace: PRESENTATION_FONTS.body,
      fontSize,
      color: theme.body,
      breakLine: false,
      valign: "top",
      margin: 0.08,
      paraSpaceAfterPt: 10,
      fit: "shrink",
    });
  }
  if (definition.notes) slide.addNotes(definition.notes);
}

function validateSlide(raw) {
  const slide = plainObject(raw);
  const title = requiredString(slide.title, 300);
  const bullets = boundedArray(slide.bullets, 20, true)
    .map((bullet) => requiredString(bullet, 2_000));
  const notes = optionalString(slide.notes, 10_000);
  return { title, bullets, notes };
}
