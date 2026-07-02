import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

export const studioTheme = EditorView.theme(
  {
    "&": {
      backgroundColor: "var(--bg1)",
      color: "var(--text)",
      fontSize: "13.5px",
      height: "100%",
    },
    ".cm-content": {
      fontFamily: "var(--font-mono)",
      caretColor: "var(--accent)",
      padding: "12px 0",
    },
    ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--accent)" },
    "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground":
      { backgroundColor: "#1c2a45 !important" },
    ".cm-activeLine": { backgroundColor: "#12182377" },
    ".cm-activeLineGutter": { backgroundColor: "transparent", color: "var(--muted)" },
    ".cm-gutters": {
      backgroundColor: "var(--bg1)",
      color: "#3d4654",
      border: "none",
      fontFamily: "var(--font-mono)",
    },
    ".cm-lineNumbers .cm-gutterElement": { padding: "0 12px 0 16px" },
    ".cm-matchingBracket": {
      backgroundColor: "#e8b33922",
      outline: "1px solid #e8b33955",
    },
    ".cm-selectionMatch": { backgroundColor: "#58a6ff22" },
    ".cm-tooltip": {
      backgroundColor: "var(--bg2)",
      border: "1px solid var(--border)",
      borderRadius: "6px",
      color: "var(--text)",
      fontFamily: "var(--font-ui)",
      fontSize: "12.5px",
    },
    ".cm-tooltip.cm-tooltip-autocomplete > ul": {
      fontFamily: "var(--font-mono)",
      fontSize: "12.5px",
    },
    ".cm-tooltip-autocomplete ul li[aria-selected]": {
      backgroundColor: "#1c2a45",
      color: "var(--text)",
    },
    ".cm-diagnostic": {
      padding: "4px 8px",
      fontFamily: "var(--font-ui)",
    },
    ".cm-diagnostic-error": { borderLeft: "3px solid var(--red)" },
    ".cm-diagnostic-warning": { borderLeft: "3px solid var(--accent)" },
    ".cm-lintRange-error": {
      backgroundImage: "none",
      textDecoration: "underline wavy var(--red) 1px",
      textUnderlineOffset: "3px",
    },
    ".cm-lintRange-warning": {
      backgroundImage: "none",
      textDecoration: "underline wavy var(--accent) 1px",
      textUnderlineOffset: "3px",
    },
    ".cm-gutter-lint .cm-gutterElement": { padding: "0 2px" },
    ".cm-panels": {
      backgroundColor: "var(--bg2)",
      color: "var(--text)",
      borderTop: "1px solid var(--border)",
    },
    ".cm-searchMatch": { backgroundColor: "#e8b33933" },
    ".cm-searchMatch.cm-searchMatch-selected": { backgroundColor: "#e8b33966" },
    ".cm-scroller": { fontFamily: "var(--font-mono)", lineHeight: "1.55" },
  },
  { dark: true },
);

export const studioHighlight = syntaxHighlighting(
  HighlightStyle.define([
    { tag: t.keyword, color: "#ff7b72" },
    { tag: t.operatorKeyword, color: "#ff7b72" },
    { tag: t.modifier, color: "#d2a8ff" },
    { tag: t.string, color: "#a5d6ff" },
    { tag: t.regexp, color: "#96d0ff" },
    { tag: t.comment, color: "#616e7e", fontStyle: "italic" },
    { tag: t.number, color: "#79c0ff" },
    { tag: t.bool, color: "#79c0ff" },
    { tag: t.special(t.variableName), color: "#e8b339" },
    { tag: t.definition(t.typeName), color: "#7ee787", fontWeight: "600" },
    { tag: t.className, color: "#ffa657" },
    { tag: t.standard(t.variableName), color: "#ffa657" },
    { tag: t.atom, color: "#56d4dd" },
    { tag: t.meta, color: "#8b949e" },
    { tag: t.operator, color: "#8b98a9" },
    { tag: t.brace, color: "#8b98a9" },
    { tag: t.punctuation, color: "#8b98a9" },
    { tag: t.invalid, color: "#f85149", textDecoration: "underline" },
    { tag: t.variableName, color: "#dce3ec" },
  ]),
);
