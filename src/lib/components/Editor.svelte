<script lang="ts">
  import { onMount } from "svelte";
  import {
    EditorView,
    keymap,
    lineNumbers,
    highlightActiveLine,
    highlightActiveLineGutter,
    highlightSpecialChars,
    drawSelection,
  } from "@codemirror/view";
  import { EditorState } from "@codemirror/state";
  import {
    defaultKeymap,
    history,
    historyKeymap,
    indentWithTab,
  } from "@codemirror/commands";
  import { bracketMatching, indentOnInput, indentUnit } from "@codemirror/language";
  import { closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
  import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
  import { linter, lintGutter, lintKeymap } from "@codemirror/lint";
  import { yara, yaraCompletion } from "$lib/yara/language";
  import { studioHighlight, studioTheme } from "$lib/yara/cm-theme";
  import { validateRules } from "$lib/api";
  import { app } from "$lib/state.svelte";

  let container: HTMLDivElement;

  const yaraLinter = linter(
    async (view) => {
      const result = await validateRules(view.state.doc.toString());
      app.validation = result;
      const docLength = view.state.doc.length;

      return [...result.errors, ...result.warnings].map((d) => {
        let from = Math.min(d.from, docLength);
        let to = Math.min(Math.max(d.to, from), docLength);
        if (from === to) {
          if (to < docLength) to += 1;
          else if (from > 0) from -= 1;
        }
        return {
          from,
          to,
          severity: d.level,
          message: d.detail ? `${d.title} — ${d.detail}` : d.title,
        };
      });
    },
    { delay: 300 },
  );

  onMount(() => {
    const view = new EditorView({
      state: EditorState.create({
        doc: app.source,
        extensions: [
          lineNumbers(),
          highlightActiveLineGutter(),
          highlightSpecialChars(),
          history(),
          drawSelection(),
          EditorState.allowMultipleSelections.of(true),
          indentOnInput(),
          indentUnit.of("    "),
          bracketMatching(),
          closeBrackets(),
          highlightActiveLine(),
          highlightSelectionMatches(),
          keymap.of([
            ...closeBracketsKeymap,
            ...defaultKeymap,
            ...searchKeymap,
            ...historyKeymap,
            ...completionKeymap,
            ...lintKeymap,
            indentWithTab,
          ]),
          yara,
          yaraCompletion(),
          yaraLinter,
          lintGutter(),
          studioTheme,
          studioHighlight,
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              app.source = update.state.doc.toString();
            }
            if (update.selectionSet || update.docChanged) {
              const head = update.state.selection.main.head;
              const line = update.state.doc.lineAt(head);
              app.cursor = { line: line.number, column: head - line.from + 1 };
            }
          }),
        ],
      }),
      parent: container,
    });

    return () => view.destroy();
  });
</script>

<div class="editor" bind:this={container}></div>

<style>
  .editor {
    height: 100%;
    overflow: hidden;
  }
  .editor :global(.cm-editor) {
    height: 100%;
  }
  .editor :global(.cm-editor.cm-focused) {
    outline: none;
  }
</style>
