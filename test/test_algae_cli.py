from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "algae.py"

sys.path.insert(0, str(ROOT))

from algae.ast import AxiomDecl, LetDecl, OpDecl, SortDecl, VarDecl  # noqa: E402
from algae.format import format_spec  # noqa: E402
from algae.parser import parse_text  # noqa: E402


def semantic_payload(module) -> list[tuple]:
    # The declaration payloads that formatting must preserve, ignoring line
    # numbers and comments. Nodes are dataclasses, so == is structural.
    payload: list[tuple] = []
    for decl in module.declarations:
        if isinstance(decl, SortDecl):
            payload.append(("sort", decl.names, decl.values))
        elif isinstance(decl, OpDecl):
            payload.append(("op", decl.name, decl.domain, decl.codomain))
        elif isinstance(decl, VarDecl):
            payload.append(("var", decl.name, decl.sort))
        elif isinstance(decl, AxiomDecl):
            payload.append(("axiom", decl.expr))
        elif isinstance(decl, LetDecl):
            payload.append(("let", decl.name, decl.expr))
    return payload


class AlgaeCliTests(unittest.TestCase):
    def run_cli(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(CLI), *args],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def check_source(self, source: str) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "spec.alg"
            path.write_text(source, encoding="utf-8")
            return self.run_cli("check", str(path))

    def test_check_accepts_equational_fixtures(self) -> None:
        result = self.run_cli("check", "test/stack.alg", "test/kvstore.alg", "test/base/container.alg")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("test/stack.alg: ok", result.stdout)
        self.assertIn("test/kvstore.alg: ok", result.stdout)
        self.assertIn("test/base/container.alg: ok", result.stdout)

    def test_check_rejects_old_and_malformed_syntax(self) -> None:
        paths = sorted(str(path.relative_to(ROOT)) for path in (ROOT / "test/reject").glob("*.alg"))
        result = self.run_cli("check", *paths)

        self.assertEqual(result.returncode, 1)
        self.assertIn("Expected", result.stdout)
        self.assertIn("found", result.stdout)
        self.assertNotIn(": ok", result.stdout)

    def test_print_outputs_json_ast(self) -> None:
        result = self.run_cli("print", "test/stack.alg")

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["ast"]["kind"], "Module")
        self.assertEqual(payload["ast"]["declarations"][0]["kind"], "SortDecl")
        self.assertEqual(payload["ast"]["declarations"][2]["kind"], "OpDecl")

    def test_fmt_converts_ascii_aliases_to_unicode(self) -> None:
        source = "\n".join(
            [
                "sort Stack, Elem;",
                "sort Error = {empty_error};",
                "op empty : arrow Stack;",
                "op push : Stack product Elem arrow Stack;",
                "op pop : Stack arrow Stack | Error;",
                "var s : Stack;",
                "var e : Elem;",
                "axiom pop(push(s, e)) neq s;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "alias.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", str(path))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("op empty : → Stack;", result.stdout)
        self.assertIn("Stack × Elem → Stack", result.stdout)
        self.assertIn("pop(push(s, e)) ≠ s", result.stdout)

    def test_fmt_ascii_outputs_keyword_aliases(self) -> None:
        result = self.run_cli("fmt", "--ascii", "test/stack.alg")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Stack product Elem arrow Stack", result.stdout)
        self.assertIn("Stack arrow Stack product Elem | Error", result.stdout)
        self.assertNotIn("→", result.stdout)
        self.assertNotIn("×", result.stdout)

    def test_let_expression_parses_and_formats(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "op f : S -> S;",
                "var x : S;",
                "axiom let y = f(x) in let z = f(y) in f(z) = x;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "let.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stderr)
        self.assertEqual(fmt_result.returncode, 0, fmt_result.stderr)
        self.assertIn(
            "axiom let y = f(x) in\n      let z = f(y) in\n      f(z) = x;",
            fmt_result.stdout,
        )

    def test_toplevel_let_parses_and_formats(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "op f : S -> S;",
                "var x : S;",
                "let y = f(x);",
                "let z = f(y);",
                "axiom f(z) = x;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "toplevel_let.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))
            print_result = self.run_cli("print", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stderr)
        self.assertEqual(fmt_result.returncode, 0, fmt_result.stderr)
        self.assertIn("let y = f(x);", fmt_result.stdout)
        self.assertIn("let z = f(y);", fmt_result.stdout)
        payload = json.loads(print_result.stdout)
        self.assertEqual(payload["ast"]["declarations"][3]["kind"], "LetDecl")
        self.assertEqual(payload["ast"]["declarations"][3]["name"], "y")

    def test_application_sugar_parses_and_formats(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "op f : S × S -> S;",
                "var x : S;",
                "axiom x.f(x).f(x) = f(f(x, x), x);",
                "axiom x |> f(x) = f(x, x);",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "sugar.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", "--no-valign", str(path))
            ascii_result = self.run_cli("fmt", "--no-valign", "--ascii", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stderr)
        self.assertIn("axiom x.f(x).f(x) = f(f(x, x), x);", fmt_result.stdout)
        self.assertIn("axiom x ▷ f(x) = f(x, x);", fmt_result.stdout)
        self.assertIn("axiom x |> f(x) = f(x, x);", ascii_result.stdout)

    def test_fmt_aligns_colons_unless_disabled(self) -> None:
        result = self.run_cli("fmt", "test/stack.alg")
        plain = self.run_cli("fmt", "--no-valign", "test/stack.alg")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("op empty : → Stack;", result.stdout)
        self.assertIn("op push  : Stack × Elem → Stack;", result.stdout)
        self.assertIn("op pop   : Stack → Stack × Elem | Error;", result.stdout)
        self.assertIn("axiom empty().pop   = empty_error;", result.stdout)
        self.assertIn("op push : Stack × Elem → Stack;", plain.stdout)
        self.assertIn("axiom empty().pop = empty_error;", plain.stdout)

    def test_destructuring_let_parses_and_formats(self) -> None:
        source = "\n".join(
            [
                "sort S, T;",
                "op pair : S -> S × T;",
                "var x : S;",
                "axiom let (a, _) = pair(x) in a = x;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "destructure.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stdout)
        self.assertIn(
            "axiom let (a, _) = pair(x) in\n      a = x;",
            fmt_result.stdout,
        )

    def test_check_reports_type_errors(self) -> None:
        source = "\n".join(
            [
                "sort Stack, Elem;",
                "sort Error = {empty_error};",
                "op pop : Stack -> Stack × Elem | Error;",
                "var s : Stack;",
                "axiom let (rest, x) = pop(s) in rest = s;",
                "axiom missing(s) = s;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "typed.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("check", str(path))
            loose = self.run_cli("check", "--syntax-only", str(path))

        self.assertEqual(result.returncode, 1)
        self.assertIn("type error at line 5, cannot destructure sum type", result.stdout)
        self.assertIn("type error at line 6, undeclared identifier missing", result.stdout)
        self.assertEqual(loose.returncode, 0, loose.stdout)
        self.assertIn(": ok", loose.stdout)

    def test_fmt_preserves_comments(self) -> None:
        source = "\n".join(
            [
                "# A simple stack.",
                "sort Stack;",
                "op empty : arrow Stack;  # constructor",
                "var s : Stack;",
                "axiom empty() = s;",
                "# end of spec",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "commented.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", str(path))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("# A simple stack.\nsort Stack;", result.stdout)
        self.assertIn("op empty : → Stack;  # constructor", result.stdout)
        self.assertIn("# end of spec", result.stdout)

    def test_fmt_inplace_rewrites_file(self) -> None:
        source = "sort Stack,Elem;op empty:arrow Stack;var s:Stack;axiom empty()=s;\n"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "stack.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", "--inplace", str(path))
            rewritten = path.read_text(encoding="utf-8")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertIn("sort Stack, Elem;", rewritten)
        self.assertIn("op empty : → Stack;", rewritten)
        self.assertIn("axiom empty() = s;", rewritten)


    def test_check_rejects_negative_literal_for_natural(self) -> None:
        result = self.check_source("var n : ℕ;\naxiom n = -1;\n")

        self.assertEqual(result.returncode, 1)
        self.assertIn("cannot equate ℕ with ℤ", result.stdout)

    def test_check_accepts_negative_literal_for_integer(self) -> None:
        result = self.check_source("var z : ℤ;\naxiom z = -1;\n")

        self.assertEqual(result.returncode, 0, result.stdout)

    def test_check_rejects_natural_subtraction_for_natural(self) -> None:
        result = self.check_source("var n : ℕ;\naxiom n = 1 - 2;\n")

        self.assertEqual(result.returncode, 1)
        self.assertIn("cannot equate ℕ with ℤ", result.stdout)

    def test_check_subtraction_widens_only_naturals(self) -> None:
        source = "\n".join(
            [
                "var z : ℤ;",
                "var n : ℕ;",
                "axiom z = 1 - 2;",  # ℕ - ℕ is ℤ
                "axiom z = z - 1;",  # ℤ - ℕ stays ℤ
                "axiom n = 1 + 2;",  # other arithmetic still ℕ
                "axiom n = 2 * 3;",
                "",
            ]
        )
        result = self.check_source(source)

        self.assertEqual(result.returncode, 0, result.stdout)

    def test_check_rejects_duplicate_destructuring_binders(self) -> None:
        source = "\n".join(
            [
                "sort S, T;",
                "op pair : S × T → S × T;",
                "var s : S;",
                "var t : T;",
                "axiom let (x, x) = pair(s, t) in x = t;",
                "",
            ]
        )
        result = self.check_source(source)

        self.assertEqual(result.returncode, 1)
        self.assertIn("type error at line 5, duplicate binder x in destructuring pattern", result.stdout)

    def test_check_accepts_repeated_wildcard_binders(self) -> None:
        source = "\n".join(
            [
                "sort S, T;",
                "op triple : → S × T × S;",
                "var s : S;",
                "axiom let (_, _, x) = triple() in x = s;",
                "",
            ]
        )
        result = self.check_source(source)

        self.assertEqual(result.returncode, 0, result.stdout)

    def test_check_rejects_implicit_error_narrowing(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "sort Error = {oops};",
                "op f : → S | Error;",
                "op g : S → S;",
                "var x : S;",
                "axiom g(f()) = x;",
                "",
            ]
        )
        result = self.check_source(source)

        self.assertEqual(result.returncode, 1)
        self.assertIn("no signature of g matches (S | Error)", result.stdout)

    def test_check_accepts_cast_convention_for_narrowing(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "sort Error = {oops};",
                "op f : → S | Error;",
                "op g : S → S;",
                "op cast : (S | Error) → S;",
                "var x : S;",
                "axiom g(cast(f())) = x;",
                "",
            ]
        )
        result = self.check_source(source)

        self.assertEqual(result.returncode, 0, result.stdout)

    def test_fmt_preserves_type_parentheses(self) -> None:
        source = "\n".join(
            [
                "sort A, B, C, D;",
                "op f : → A × (B | C);",
                "op g : (A → B) × C → D;",
                "var v : (A × B) × C;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "types.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", "--no-valign", str(path))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("op f : → A × (B | C);", result.stdout)
        self.assertIn("op g : (A → B) × C → D;", result.stdout)
        self.assertIn("var v : (A × B) × C;", result.stdout)

    def test_fmt_preserves_expression_parentheses(self) -> None:
        source = "\n".join(
            [
                "var a : ℕ;",
                "var b : ℕ;",
                "var c : ℕ;",
                "var d : ℕ;",
                "axiom (a + b) * c = d;",
                "axiom a + b * c = d;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "exprs.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", "--no-valign", str(path))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("axiom (a + b) * c = d;", result.stdout)
        self.assertIn("axiom a + b * c = d;", result.stdout)

    def test_fmt_round_trips_grouping(self) -> None:
        sources = [
            "axiom (a + b) * c = d;",
            "axiom a - (b - c) = d;",
            "axiom (a ⟹ b) ⟹ c;",
            "axiom a ⟹ b ⟹ c;",
            "axiom ¬ (a ∧ b) ∨ c;",
            "axiom (a ∨ b) ∧ c;",
            "axiom (a + b).f = c;",
            "axiom a ▷ f(b) = c;",
            "axiom (a + b)' = c;",
            "axiom x = (if a then b else c) + 1;",
            "axiom (let y = f(x) in y) = z;",
            "axiom - (a + b) = c;",
            "axiom (- a) * b = c;",
            "axiom a * (- b) = c;",
            "op f : → A × (B | C);",
            "op g : (A → B) × C → D;",
            "var v : (A × B) × C;",
            "var w : A | (B | C);",
            "var q : Seq[A | B];",
        ]
        for source in sources:
            with self.subTest(source=source):
                module = parse_text(source)
                rendered = format_spec(module)
                reparsed = parse_text(rendered)
                self.assertEqual(semantic_payload(reparsed), semantic_payload(module), rendered)
                self.assertEqual(format_spec(reparsed), rendered)


if __name__ == "__main__":
    unittest.main()
