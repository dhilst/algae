# Sphinx configuration for the Algae documentation site.
#
# The site prose (tutorial.md) lives here in docs/ as its own source. The full
# language specification (lang-specs/spec.md) stays in the repo root and is
# intentionally not published here. The interactive editors are wired up by
# _static/algae-init.js, which loads the algae-wasm ESM (algae_wasm.js +
# algae_wasm_bg.wasm) and the bundled CodeMirror editor (algae-editor.js) — all
# three are copied into _static/ by docs/build.sh (locally) or the CI `docs`
# job. If those assets are absent the pages still render with static Pygments
# highlighting.

from pygments.lexer import RegexLexer, words
from pygments.token import (
    Comment, Keyword, Name, Number, Operator, Punctuation, Text,
)

# --- Project metadata -------------------------------------------------------
project = "Algae"
copyright = "2026, Algae contributors"
author = "Algae contributors"

# --- General configuration --------------------------------------------------
extensions = ["myst_parser"]

myst_enable_extensions = ["colon_fence", "deflist"]
myst_heading_anchors = 3

source_suffix = {".md": "markdown", ".rst": "restructuredtext"}
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store", "README.md"]

# --- HTML output ------------------------------------------------------------
html_theme = "furo"
html_title = "Algae"
html_static_path = ["_static"]
# Loaded as an ES module so it can `import` the wasm + editor bundles.
html_js_files = [("algae-init.js", {"type": "module"})]


# --- A minimal Pygments lexer for `.alg` fenced blocks ----------------------
# This only provides the static (pre-JS / no-JS) highlighting fallback; once the
# page's JavaScript runs, each block is replaced by a CodeMirror editor.
class AlgLexer(RegexLexer):
    name = "Algae"
    aliases = ["alg", "algae"]
    filenames = ["*.alg"]

    tokens = {
        "root": [
            (r"#.*$", Comment.Single),
            (r"-{24,}|─{24,}", Punctuation),  # inference separator
            (words((
                "import", "sort", "op", "axiom", "rule", "lemma", "theorem",
                "theory", "law", "model", "include", "end", "proof", "qed",
                "by", "case", "cases", "props", "laws", "satisfies", "iff",
                "as", "forall", "exists", "st", "lambda",
            ), suffix=r"\b"), Keyword),
            (r"\bwip\b", Keyword.Pseudo),
            (words(("Sort", "Prop"), suffix=r"\b"), Keyword.Type),
            (r"\bFalse\b", Name.Constant),
            (r"[0-9]+", Number),
            (r"⊢|\|-|->|→|=>|⇒|<=>|⇔|/\\|∧|\\/|"
             r"∨|~|¬|∀|∃|λ|×|:=|==|<=|>=|[+\-*/<>=|]",
             Operator),
            (r"[A-Za-z_][A-Za-z0-9_]*", Name),
            (r"[(),;:.]", Punctuation),
            (r"\s+", Text),
            (r".", Text),
        ],
    }


def setup(app):
    app.add_lexer("alg", AlgLexer)
    return {"parallel_read_safe": True, "parallel_write_safe": True}
