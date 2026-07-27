import {
  PDFArray,
  PDFBool,
  PDFDict,
  PDFHexString,
  PDFName,
  PDFNumber,
  PDFOperator,
  PDFOperatorNames,
} from "@cantoo/pdf-lib";
import { rejectOffice } from "../errors.mjs";

export function createTaggedText(document, maximumItems) {
  const context = document.context;
  const root = PDFDict.withContext(context);
  const rootRef = context.register(root);
  const rootChildren = PDFArray.withContext(context);
  const parentNumbers = PDFArray.withContext(context);
  const parentTree = PDFDict.withContext(context);
  const parentTreeRef = context.register(parentTree);
  const pageStates = new Map();
  let itemCount = 0;

  root.set(PDFName.of("Type"), PDFName.of("StructTreeRoot"));
  root.set(PDFName.of("K"), rootChildren);
  root.set(PDFName.of("ParentTree"), parentTreeRef);
  root.set(PDFName.of("ParentTreeNextKey"), PDFNumber.of(0));
  parentTree.set(PDFName.of("Nums"), parentNumbers);
  document.catalog.set(PDFName.of("StructTreeRoot"), rootRef);
  document.catalog.set(PDFName.of("MarkInfo"), markInfo(context));

  function begin(page, text) {
    if (itemCount >= maximumItems) rejectOffice("output_too_large");
    const state = pageState(page);
    const mcid = state.children.size();
    const actualText = PDFHexString.fromText(text);
    const element = structureElement(
      context,
      rootRef,
      page.ref,
      mcid,
      actualText,
    );
    const elementRef = context.register(element);
    rootChildren.push(elementRef);
    state.children.push(elementRef);
    itemCount += 1;
    return markedContent(context, mcid, actualText);
  }

  function pageState(page) {
    const existing = pageStates.get(page.ref);
    if (existing) return existing;
    const key = pageStates.size;
    const children = PDFArray.withContext(context);
    page.node.set(PDFName.of("StructParents"), PDFNumber.of(key));
    parentNumbers.push(PDFNumber.of(key));
    parentNumbers.push(children);
    root.set(PDFName.of("ParentTreeNextKey"), PDFNumber.of(key + 1));
    const state = Object.freeze({ children });
    pageStates.set(page.ref, state);
    return state;
  }

  return Object.freeze({ begin });
}

function structureElement(context, parent, page, mcid, actualText) {
  const element = PDFDict.withContext(context);
  element.set(PDFName.of("Type"), PDFName.of("StructElem"));
  element.set(PDFName.of("S"), PDFName.of("Span"));
  element.set(PDFName.of("P"), parent);
  element.set(PDFName.of("Pg"), page);
  element.set(PDFName.of("K"), PDFNumber.of(mcid));
  element.set(PDFName.of("ActualText"), actualText);
  return element;
}

function markedContent(context, mcid, actualText) {
  const properties = PDFDict.withContext(context);
  properties.set(PDFName.of("MCID"), PDFNumber.of(mcid));
  properties.set(PDFName.of("ActualText"), actualText);
  return PDFOperator.of(PDFOperatorNames.BeginMarkedContentSequence, [
    PDFName.of("Span"),
    properties,
  ]);
}

function markInfo(context) {
  const info = PDFDict.withContext(context);
  info.set(PDFName.of("Marked"), PDFBool.True);
  return info;
}
