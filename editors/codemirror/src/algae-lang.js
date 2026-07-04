// CodeMirror 6 language support for Algae (.alg).
//
// A token-level `StreamLanguage` hand-ported from the tree-sitter grammar at
// editors/tree-sitter/grammar.js and its highlight query
// editors/tree-sitter/queries/highlights.scm. It is intentionally NOT a full
// parser: it recognizes comments, keywords (with sub-classes), ASCII + Unicode
// operators, the 24+-dash inference separator, numbers, holes, and identifiers,
// which is ample for documentation snippets. Highlighting and proof checking
// share one source of truth for *checking* (algae-wasm); this file only styles.

import { StreamLanguage, LanguageSupport, HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

// --- Keyword tables (spec section 2.5) --------------------------------------

// Structural / declaration keywords.
const KW_STRUCT = new Set([
  "import", "sort", "op", "axiom", "rule", "lemma", "theorem",
  "theory", "law", "model", "include", "end",
]);
// Proof keywords.
const KW_PROOF = new Set(["proof", "qed", "by", "case", "cases", "then", "props", "laws"]);
// Modifier-ish keywords.
const KW_MOD = new Set(["satisfies", "iff", "as"]);
// Binder / quantifier keywords (ASCII spellings; Unicode ∀ ∃ λ handled below).
const KW_BINDER = new Set(["forall", "exists", "st", "lambda"]);
// Built-in types and the False constant.
const TYPE_BUILTIN = new Set(["Sort", "Prop"]);
const CONST_BUILTIN = new Set(["False"]);

// Keywords after which the next identifier names a *declaration*.
const DECL_DEF = new Set(["sort", "op", "axiom", "rule", "lemma", "theorem", "law"]);
const DECL_TYPE = new Set(["theory", "model", "include"]);

// Unicode operator glyphs that are single code points.
const UNI_BINDER = new Set(["∀", "∃", "λ"]);
const UNI_LOGIC = new Set(["¬", "∧", "∨", "⇒", "⇔", "⊢"]);
const UNI_OP = new Set(["→", "×"]);

const IDENT_START = /[A-Za-z_]/;
const IDENT_CHAR = /[A-Za-z0-9_]/;

function startState() {
  // `expect` colours the next identifier: "def" (value/proof name),
  // "type" (theory/model name), "module" (import target), or null.
  return { expect: null };
}

function token(stream, state) {
  if (stream.eatSpace()) return null;

  const ch = stream.peek();

  // Line comments: `# ... EOL`.
  if (ch === "#") {
    stream.skipToEnd();
    return "comment";
  }

  // Inference separator: 24+ dashes (ASCII `-` or box-drawing `─`).
  if (stream.match(/^-{24,}/) || stream.match(/^─{24,}/)) {
    return "separator";
  }

  // Numbers (also usable as operator names, e.g. `op 0 : -> Nat`).
  if (/[0-9]/.test(ch)) {
    stream.match(/^[0-9]+/);
    if (state.expect === "def") { state.expect = null; return "def"; }
    return "number";
  }

  // Identifiers / keywords / holes.
  if (IDENT_START.test(ch)) {
    let word = "";
    while (!stream.eol() && IDENT_CHAR.test(stream.peek())) word += stream.next();

    if (word === "_") { return "hole"; }
    if (word === "wip") { state.expect = null; return "wip"; }

    // A pending declaration name wins over the generic identifier colour,
    // but reserved words are never treated as names.
    const reserved =
      KW_STRUCT.has(word) || KW_PROOF.has(word) || KW_MOD.has(word) ||
      KW_BINDER.has(word) || TYPE_BUILTIN.has(word) || CONST_BUILTIN.has(word);

    if (state.expect && !reserved) {
      const kind = state.expect;
      state.expect = null;
      return kind === "module" ? "module" : kind === "type" ? "typeDef" : "def";
    }

    if (KW_STRUCT.has(word)) {
      if (word === "import") state.expect = "module";
      else if (DECL_DEF.has(word)) state.expect = "def";
      else if (DECL_TYPE.has(word)) state.expect = "type";
      else state.expect = null;
      return "keyword";
    }
    if (KW_PROOF.has(word)) { state.expect = null; return "keywordProof"; }
    if (KW_MOD.has(word)) return "keywordMod";
    if (KW_BINDER.has(word)) return "binder";
    if (TYPE_BUILTIN.has(word)) return "typeBuiltin";
    if (CONST_BUILTIN.has(word)) return "constBuiltin";

    return "variable";
  }

  // Single-code-point Unicode operators.
  if (UNI_BINDER.has(ch)) { stream.next(); return "binder"; }
  if (UNI_LOGIC.has(ch)) { stream.next(); return "logic"; }
  if (UNI_OP.has(ch)) { stream.next(); return "operator"; }

  // Multi-character ASCII operators (longest first).
  if (stream.match("<=>")) return "logic";      // biconditional
  if (stream.match("|-")) return "logic";        // turnstile
  if (stream.match("=>")) return "logic";        // implication
  if (stream.match("->")) return "operator";     // function arrow
  if (stream.match(":=")) return "operator";     // proof binding
  if (stream.match("/\\")) return "logic";       // conjunction
  if (stream.match("\\/")) return "logic";       // disjunction
  if (stream.match("==")) return "operator";
  if (stream.match("<=")) return "operator";
  if (stream.match(">=")) return "operator";

  // Single-character operators and punctuation.
  if ("~".includes(ch)) { stream.next(); return "logic"; }        // negation
  if ("+-*/<>=".includes(ch)) { stream.next(); return "operator"; }
  if (ch === "|") { stream.next(); return "operator"; }           // sum type
  if (ch === ".") { stream.next(); return "punctuation"; }
  if ("(),;:".includes(ch)) { stream.next(); return "punctuation"; }

  // Anything else: consume one char so we never stall.
  stream.next();
  return null;
}

// Map our token names to @lezer/highlight tags.
const tokenTable = {
  comment: t.lineComment,
  keyword: t.keyword,
  keywordProof: t.controlKeyword,
  keywordMod: t.modifier,
  binder: t.operatorKeyword,
  wip: t.invalid,
  typeBuiltin: t.standard(t.typeName),
  constBuiltin: t.bool,
  number: t.number,
  operator: t.operator,
  logic: t.logicOperator,
  separator: t.contentSeparator,
  hole: t.atom,
  def: t.definition(t.variableName),
  typeDef: t.definition(t.typeName),
  module: t.namespace,
  variable: t.variableName,
  punctuation: t.punctuation,
};

export const algaeStreamLanguage = StreamLanguage.define({
  name: "algae",
  startState,
  token,
  tokenTable,
  languageData: {
    commentTokens: { line: "#" },
  },
});

// A theme-agnostic highlight style. Colours are chosen to read on both light
// and dark backgrounds; the editor theme sets the surrounding surface.
export const algaeHighlightStyle = HighlightStyle.define([
  { tag: t.lineComment, color: "#6a737d", fontStyle: "italic" },
  { tag: t.keyword, color: "#a626a4" },
  { tag: t.controlKeyword, color: "#c18401", fontWeight: "bold" },
  { tag: t.modifier, color: "#a626a4" },
  { tag: t.operatorKeyword, color: "#0184bc", fontWeight: "bold" },
  { tag: t.invalid, color: "#e45649", fontWeight: "bold", textDecoration: "underline" },
  { tag: t.standard(t.typeName), color: "#c18401" },
  { tag: t.typeName, color: "#c18401" },
  { tag: t.bool, color: "#986801" },
  { tag: t.number, color: "#986801" },
  { tag: t.operator, color: "#4078f2" },
  { tag: t.logicOperator, color: "#0184bc", fontWeight: "bold" },
  { tag: t.contentSeparator, color: "#a0a1a7" },
  { tag: t.atom, color: "#0184bc" },
  { tag: t.definition(t.variableName), color: "#4078f2", fontWeight: "bold" },
  { tag: t.definition(t.typeName), color: "#c18401", fontWeight: "bold" },
  { tag: t.namespace, color: "#50a14f" },
  { tag: t.variableName, color: "#383a42" },
  { tag: t.punctuation, color: "#696c77" },
]);

// The full language extension: grammar + highlight style.
export function algae() {
  return new LanguageSupport(algaeStreamLanguage, [
    syntaxHighlighting(algaeHighlightStyle),
  ]);
}
