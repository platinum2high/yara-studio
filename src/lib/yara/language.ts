import { StreamLanguage, type StreamParser, type StringStream } from "@codemirror/language";
import {
  autocompletion,
  completeFromList,
  snippetCompletion,
  type Completion,
} from "@codemirror/autocomplete";

const DECLARATIONS = ["rule", "private", "global", "import", "include"];
const SECTIONS = ["meta", "strings", "condition"];
const OPERATORS = [
  "and", "or", "not", "defined",
  "matches", "contains", "icontains",
  "startswith", "istartswith", "endswith", "iendswith", "iequals",
  "of", "for", "all", "any", "none", "them", "in", "at",
];
const CONSTANTS = ["true", "false", "filesize", "entrypoint"];
const FUNCTIONS = [
  "uint8", "uint16", "uint32", "int8", "int16", "int32",
  "uint8be", "uint16be", "uint32be", "int8be", "int16be", "int32be",
];
const MODIFIERS = [
  "nocase", "wide", "ascii", "fullword", "xor", "base64", "base64wide",
];
const MODULES = [
  "pe", "elf", "macho", "dotnet", "math", "hash", "string", "time",
  "console", "lnk", "dex", "magic", "cuckoo", "vt", "crx",
];

const wordSets = new Map<string, string>();
for (const w of DECLARATIONS) wordSets.set(w, "keyword");
for (const w of SECTIONS) wordSets.set(w, "keyword");
for (const w of OPERATORS) wordSets.set(w, "operatorKeyword");
for (const w of CONSTANTS) wordSets.set(w, "bool");
for (const w of FUNCTIONS) wordSets.set(w, "variableName.standard");
for (const w of MODIFIERS) wordSets.set(w, "modifier");
for (const w of MODULES) wordSets.set(w, "className");

interface YaraState {
  inBlockComment: boolean;
  inHexString: boolean;
  afterEquals: boolean;
  expectRuleName: boolean;
  afterRuleName: boolean;
}

function tokenHex(stream: StringStream, state: YaraState): string | null {
  if (stream.eatSpace()) return null;
  if (stream.match("//")) {
    stream.skipToEnd();
    return "comment";
  }
  if (stream.match("/*")) {
    state.inBlockComment = true;
    return "comment";
  }
  if (stream.eat("}")) {
    state.inHexString = false;
    return "brace";
  }
  if (stream.match(/^\[\s*\d*\s*-?\s*\d*\s*\]/)) return "meta";
  if (stream.eat("(") || stream.eat(")") || stream.eat("|") || stream.eat("~")) {
    return "operator";
  }
  if (stream.match(/^[0-9A-Fa-f?]{2}/)) {
    return stream.current().includes("?") ? "atom" : "number";
  }
  stream.next();
  return "invalid";
}

function tokenString(stream: StringStream): string {
  while (!stream.eol()) {
    if (stream.match(/^\\[\\"'ntr]/) || stream.match(/^\\x[0-9A-Fa-f]{2}/)) continue;
    const ch = stream.next();
    if (ch === '"') break;
  }
  return "string";
}

function tokenRegex(stream: StringStream): string {
  let escaped = false;
  while (!stream.eol()) {
    const ch = stream.next();
    if (escaped) {
      escaped = false;
    } else if (ch === "\\") {
      escaped = true;
    } else if (ch === "/") {
      stream.match(/^[is]+/);
      break;
    }
  }
  return "regexp";
}

function token(stream: StringStream, state: YaraState): string | null {
  if (state.inBlockComment) {
    while (!stream.eol()) {
      if (stream.match("*/")) {
        state.inBlockComment = false;
        break;
      }
      stream.next();
    }
    return "comment";
  }

  if (state.inHexString) return tokenHex(stream, state);

  if (stream.eatSpace()) return null;

  if (stream.match("//")) {
    stream.skipToEnd();
    return "comment";
  }
  if (stream.match("/*")) {
    state.inBlockComment = true;
    return "comment";
  }

  const afterEquals = state.afterEquals;
  state.afterEquals = false;

  if (stream.eat('"')) return tokenString(stream);

  // A `/` can only start a regex in YARA: integer division uses `\`.
  if (stream.eat("/")) return tokenRegex(stream);

  if (stream.eat("{")) {
    if (afterEquals) {
      state.inHexString = true;
    } else {
      state.afterRuleName = false;
    }
    return "brace";
  }
  if (stream.eat("}")) return "brace";

  if (stream.match(/^[$#@!][A-Za-z0-9_]*\*?/)) {
    // A bare `!` is negation-adjacent punctuation, not a string length.
    if (stream.current() === "!") return "operator";
    return "variableName.special";
  }

  if (stream.match(/^0x[0-9A-Fa-f]+/)) return "number";
  if (stream.match(/^\d+(KB|MB)?/)) return "number";

  if (stream.match(/^[A-Za-z_][A-Za-z0-9_]*/)) {
    const word = stream.current();

    if (state.expectRuleName) {
      state.expectRuleName = false;
      state.afterRuleName = true;
      return "typeName.definition";
    }
    if (word === "rule") {
      state.expectRuleName = true;
      state.afterRuleName = false;
      return "keyword";
    }
    if (state.afterRuleName) return "atom";

    const style = wordSets.get(word);
    if (style === "className" && stream.peek() !== "." ) {
      // Module names only get module colouring when actually dereferenced
      // or right after `import`; otherwise they are ordinary identifiers.
      const line = stream.string.slice(0, stream.pos);
      if (!/import\s+"?[A-Za-z_]*$/.test(line)) return "variableName";
    }
    return style ?? "variableName";
  }

  if (stream.eat(":")) return "punctuation";
  if (stream.eat("=")) {
    state.afterEquals = true;
    return "operator";
  }
  if (stream.match(/^[+\-*\\%<>&|^.,()[\]]/)) return "operator";

  stream.next();
  return null;
}

export const yaraStreamParser: StreamParser<YaraState> = {
  name: "yara",
  startState: () => ({
    inBlockComment: false,
    inHexString: false,
    afterEquals: false,
    expectRuleName: false,
    afterRuleName: false,
  }),
  token,
  languageData: {
    commentTokens: { line: "//", block: { open: "/*", close: "*/" } },
    closeBrackets: { brackets: ["(", "[", "{", '"'] },
  },
};

export const yara = StreamLanguage.define(yaraStreamParser);

const KEYWORD_COMPLETIONS: Completion[] = [
  ...DECLARATIONS.map((w) => ({ label: w, type: "keyword" })),
  ...SECTIONS.map((w) => ({ label: `${w}:`, type: "keyword" })),
  ...OPERATORS.map((w) => ({ label: w, type: "keyword" })),
  ...CONSTANTS.map((w) => ({ label: w, type: "constant" })),
  ...FUNCTIONS.map((w) => ({ label: w, type: "function" })),
  ...MODIFIERS.map((w) => ({ label: w, type: "enum" })),
  ...MODULES.map((w) => ({ label: w, type: "namespace" })),
  snippetCompletion(
    'rule ${RuleName}\n{\n    meta:\n        author = "${author}"\n        description = "${description}"\n    strings:\n        $${a} = "${pattern}"\n    condition:\n        $${a}\n}',
    { label: "rule", detail: "skeleton", type: "class", boost: 1 },
  ),
];

function definedStringIdentifiers(doc: string): Completion[] {
  const ids = new Set<string>();
  for (const m of doc.matchAll(/^\s*(\$[A-Za-z0-9_]+)\s*=/gm)) ids.add(m[1]);
  return [...ids].map((label) => ({ label, type: "variable" }));
}

export function yaraCompletion() {
  return autocompletion({
    override: [
      (context) => {
        const word = context.matchBefore(/[$#@!]?[\w]*/);
        if (!word || (word.from === word.to && !context.explicit)) return null;
        const dynamic = definedStringIdentifiers(context.state.doc.toString());
        return completeFromList([...KEYWORD_COMPLETIONS, ...dynamic])(context);
      },
    ],
  });
}
