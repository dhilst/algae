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

from algae.ast import AxiomDecl, LemmaDecl, LetDecl, OpDecl, SortDecl, VarDecl  # noqa: E402
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
            payload.append(("op", decl.name, decl.domain, decl.codomain, decl.partial))
        elif isinstance(decl, VarDecl):
            payload.append(("var", decl.names, decl.sort))
        elif isinstance(decl, AxiomDecl):
            payload.append(("axiom", decl.name, decl.expr))
        elif isinstance(decl, LemmaDecl):
            payload.append(("lemma", decl.name, decl.expr, decl.proof))
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
                "axiom pop_push pop(push(s, e)) neq s;",
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
        self.assertIn("Stack * Elem arrow Stack", result.stdout)
        self.assertIn("Stack arrow Stack * Elem | Error", result.stdout)
        self.assertNotIn("→", result.stdout)
        self.assertNotIn("×", result.stdout)

    def test_symbolic_ascii_aliases_parse_and_format(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "op f : S * Nat arrow S;",
                "var s : S;",
                "var n : Nat;",
                "var z : Int;",
                "var r : Real;",
                "var b : Bool;",
                "axiom tauto b /\\ true \\/ false;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "aliases.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))
            ascii_result = self.run_cli("fmt", "--ascii", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stdout)
        self.assertEqual(fmt_result.returncode, 0, fmt_result.stderr)
        self.assertIn("op f : S × ℕ → S;", fmt_result.stdout)
        self.assertIn("var z : ℤ;", fmt_result.stdout)
        self.assertIn("var r : ℝ;", fmt_result.stdout)
        self.assertIn("var b : 𝔹;", fmt_result.stdout)
        self.assertIn("axiom tauto b ∧ true ∨ false;", fmt_result.stdout)
        self.assertIn("op f : S * Nat arrow S;", ascii_result.stdout)
        self.assertIn("var b : Bool;", ascii_result.stdout)
        self.assertIn("axiom tauto b /\\ true \\/ false;", ascii_result.stdout)

    def test_let_expression_parses_and_formats(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "op f : S -> S;",
                "var x : S;",
                "axiom chain let y = f(x) in let z = f(y) in f(z) = x;",
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
        self.assertIn("axiom chain let y = f(x) in let z = f(y) in f(z) = x;", fmt_result.stdout)

    def test_toplevel_let_parses_and_formats(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "op f : S -> S;",
                "var x : S;",
                "let y = f(x);",
                "let z = f(y);",
                "axiom roundtrip f(z) = x;",
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
                "axiom sugar_first x.f(x).f(x) = f(f(x, x), x);",
                "axiom sugar_last x |> f(x) = f(x, x);",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "sugar.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))
            ascii_result = self.run_cli("fmt", "--ascii", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stderr)
        self.assertIn("axiom sugar_first x.f(x).f(x) = f(f(x, x), x);", fmt_result.stdout)
        self.assertIn("axiom sugar_last x ▷ f(x) = f(x, x);", fmt_result.stdout)
        self.assertIn("axiom sugar_last x |> f(x) = f(x, x);", ascii_result.stdout)

    def test_fmt_does_not_pad_separators(self) -> None:
        result = self.run_cli("fmt", "test/stack.alg")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("op empty : → Stack;", result.stdout)
        self.assertIn("op push : Stack × Elem → Stack;", result.stdout)
        self.assertIn("op pop : Stack → Stack × Elem | Error;", result.stdout)
        self.assertIn("axiom empty_pop empty().pop = empty_error;", result.stdout)
        self.assertNotIn("  :", result.stdout)
        self.assertNotIn("  =", result.stdout)

    def test_destructuring_let_parses_and_formats(self) -> None:
        source = "\n".join(
            [
                "sort S, T;",
                "op pair : S -> S × T;",
                "var x : S;",
                "axiom pair_fst let (a, _) = pair(x) in a = x;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "destructure.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stdout)
        self.assertIn("axiom pair_fst let (a, _) = pair(x) in a = x;", fmt_result.stdout)

    def test_check_reports_type_errors(self) -> None:
        source = "\n".join(
            [
                "sort Stack, Elem;",
                "sort Error = {empty_error};",
                "op pop : Stack -> Stack × Elem | Error;",
                "var s : Stack;",
                "axiom pop_rest let (rest, x) = pop(s) in rest = s;",
                "axiom missing_op missing(s) = s;",
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
                "axiom empty_eq empty() = s;",
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

    def test_fmt_inplace_respells_but_preserves_layout(self) -> None:
        source = "sort Stack,Elem;op empty:arrow Stack;var s:Stack;axiom empty_eq empty()=s;\n"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "stack.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", "--inplace", str(path))
            rewritten = path.read_text(encoding="utf-8")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")
        # Only the alias is respelled; spacing and layout stay verbatim.
        self.assertEqual(
            rewritten, "sort Stack,Elem;op empty:→ Stack;var s:Stack;axiom empty_eq empty()=s;\n"
        )

    def test_fmt_preserves_whitespace_and_layout(self) -> None:
        source = "\n".join(
            [
                "sort Elem;",
                "",
                "var q             : Elem;",
                "var e, f, default : Elem;",
                "axiom  e_eq  e   =    f;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "layout.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", str(path))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, source)


    def test_var_declares_multiple_names(self) -> None:
        source = "\n".join(
            [
                "sort Elem;",
                "var e, f : Elem;",
                "axiom e_eq e = f;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "multivar.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))
            duplicate = self.check_source("sort Elem;\nvar e, e : Elem;\n")

        self.assertEqual(check_result.returncode, 0, check_result.stdout)
        self.assertIn("var e, f : Elem;", fmt_result.stdout)
        self.assertEqual(duplicate.returncode, 1)
        self.assertIn("duplicate var e", duplicate.stdout)

    def test_axiom_names_parse_check_and_format(self) -> None:
        source = "\n".join(
            [
                "sort Q;",
                "op size : Q → ℕ;",
                "op empty : Q → 𝔹;",
                "var q : Q;",
                "axiom empty_size q.empty <==> q.size = 0;",
                "axiom assoc' q.size ≥ 0;",
                "axiom tauto (q.empty ∨ ¬ q.empty) = true;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "named.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))
            print_result = self.run_cli("print", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stdout)
        self.assertIn("axiom empty_size q.empty ⟺ q.size = 0;", fmt_result.stdout)
        self.assertIn("axiom assoc' q.size ≥ 0;", fmt_result.stdout)
        declarations = json.loads(print_result.stdout)["ast"]["declarations"]
        names = [decl.get("name") for decl in declarations if decl["kind"] == "AxiomDecl"]
        self.assertEqual(names, ["empty_size", "assoc'", "tauto"])

    def test_anonymous_axiom_is_rejected(self) -> None:
        # The first identifier after `axiom` is always the (required) name.
        result = self.check_source("var b : 𝔹;\naxiom b = b;\n")

        self.assertEqual(result.returncode, 1)
        self.assertIn("Expected expression found =", result.stdout)

    def test_axiom_name_allows_paren_body_and_primes(self) -> None:
        # A required name removes the old `name(args)` call ambiguity: the
        # body may start with `(`, and names may carry trailing primes.
        result = self.check_source("sort Q;\nvar q : Q;\naxiom refl' (q, q) = (q, q');\n")
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_check_rejects_duplicate_axiom_names(self) -> None:
        result = self.check_source("var b : 𝔹;\naxiom dup b;\naxiom dup ¬ b;\n")

        self.assertEqual(result.returncode, 1)
        self.assertIn("duplicate axiom name dup", result.stdout)

    def test_partial_op_parses_checks_and_formats(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "sort Error = {oops};",
                "op f : → S | Error;",
                "op assert : S | Error ⇸ S;",
                "op coerce : S -/-> S;",
                "var x : S;",
                "axiom assert_elim f().assert = x;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "partial.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))
            ascii_result = self.run_cli("fmt", "--ascii", str(path))
            print_result = self.run_cli("print", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stdout)
        # fmt canonicalizes -/-> to ⇸; --ascii goes the other way.
        self.assertIn("op assert : S | Error ⇸ S;", fmt_result.stdout)
        self.assertIn("op coerce : S ⇸ S;", fmt_result.stdout)
        self.assertIn("op assert : S | Error -/-> S;", ascii_result.stdout)
        self.assertIn("op coerce : S -/-> S;", ascii_result.stdout)
        declarations = json.loads(print_result.stdout)["ast"]["declarations"]
        partial = {decl["name"]: decl["partial"] for decl in declarations if decl["kind"] == "OpDecl"}
        self.assertEqual(partial, {"f": False, "assert": True, "coerce": True})

    def test_op_domain_accepts_toplevel_sum(self) -> None:
        # A top-level `|` folds the domain into one sum-typed argument,
        # grouping as in codomains: A × B | C is (A × B) | C.
        source = "\n".join(
            [
                "sort A, B;",
                "sort Error = {oops};",
                "op pair : A × B → A × B | Error;",
                "op assert : A × B | Error → A × B;",
                "var a : A;",
                "var b : B;",
                "axiom assert_elim pair(a, b).assert = (a, b);",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)

        module = parse_text(source)
        domain = [decl for decl in module.declarations if isinstance(decl, OpDecl)][1].domain
        self.assertEqual(len(domain), 1)
        self.assertEqual(domain[0].kind, "type_sum")

    def test_lemma_parses_checks_formats_and_prints(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "op f : S → S;",
                "var x : S;",
                "axiom f_id f(x) = x;",
                "lemma f_twice",
                "  x.f.f = x;",
                "proof",
                "  x.f.f;",
                "  = x.f by f_id;",
                "  = x by f_id;",
                "qed;",
                "lemma f_thrice' x.f.f.f = x;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "lemmas.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))
            print_result = self.run_cli("print", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stdout)
        # fmt is token-level: the lemma and proof come back verbatim.
        self.assertEqual(fmt_result.stdout, source)
        declarations = json.loads(print_result.stdout)["ast"]["declarations"]
        lemmas = [decl for decl in declarations if decl["kind"] == "LemmaDecl"]
        self.assertEqual([lemma["name"] for lemma in lemmas], ["f_twice", "f_thrice'"])
        steps = lemmas[0]["proof"]["data"]["steps"]
        self.assertEqual(
            [step["kind"] for step in steps], ["proof_start", "proof_rewrite", "proof_rewrite"]
        )
        self.assertEqual(steps[1]["data"]["rule"], "f_id")
        self.assertIsNone(lemmas[1]["proof"])

    def test_lemmas_are_not_checked(self) -> None:
        # Phase 1: lemmas and proofs are parsed and stored only. Nonsense
        # propositions, unknown rules, and unknown identifiers all pass.
        source = "\n".join(
            [
                "sort S;",
                "var a, b : S;",
                "lemma nonsense a = b;",
                "proof",
                "  xyz;",
                "  = abc by imaginary_rule;",
                "qed;",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_check_rejects_negative_literal_for_natural(self) -> None:
        result = self.check_source("var n : ℕ;\naxiom neg n = -1;\n")

        self.assertEqual(result.returncode, 1)
        self.assertIn("cannot equate ℕ with ℤ", result.stdout)

    def test_check_accepts_negative_literal_for_integer(self) -> None:
        result = self.check_source("var z : ℤ;\naxiom neg z = -1;\n")

        self.assertEqual(result.returncode, 0, result.stdout)

    def test_check_rejects_natural_subtraction_for_natural(self) -> None:
        result = self.check_source("var n : ℕ;\naxiom sub n = 1 - 2;\n")

        self.assertEqual(result.returncode, 1)
        self.assertIn("cannot equate ℕ with ℤ", result.stdout)

    def test_check_subtraction_widens_only_naturals(self) -> None:
        source = "\n".join(
            [
                "var z : ℤ;",
                "var n : ℕ;",
                "axiom sub_widens z = 1 - 2;",  # ℕ - ℕ is ℤ
                "axiom sub_stays z = z - 1;",  # ℤ - ℕ stays ℤ
                "axiom add_nat n = 1 + 2;",  # other arithmetic still ℕ
                "axiom mul_nat n = 2 * 3;",
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
                "axiom dup_binders let (x, x) = pair(s, t) in x = t;",
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
                "axiom wildcards let (_, _, x) = triple() in x = s;",
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
                "axiom narrow g(f()) = x;",
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
                "axiom narrow g(cast(f())) = x;",
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
            result = self.run_cli("fmt", str(path))

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
                "axiom grouped (a + b) * c = d;",
                "axiom ungrouped a + b * c = d;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "exprs.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", str(path))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("axiom grouped (a + b) × c = d;", result.stdout)
        self.assertIn("axiom ungrouped a + b × c = d;", result.stdout)

    def test_fmt_round_trips_grouping(self) -> None:
        sources = [
            "axiom r (a + b) * c = d;",
            "axiom r a - (b - c) = d;",
            "axiom r (a ⟹ b) ⟹ c;",
            "axiom r a ⟹ b ⟹ c;",
            "axiom r ¬ (a ∧ b) ∨ c;",
            "axiom r (a ∨ b) ∧ c;",
            "axiom r (a + b).f = c;",
            "axiom r a ▷ f(b) = c;",
            "axiom r (a + b)' = c;",
            "axiom r x = (if a then b else c) + 1;",
            "axiom r (let y = f(x) in y) = z;",
            "axiom r - (a + b) = c;",
            "axiom r (- a) * b = c;",
            "axiom r a * (- b) = c;",
            "op f : → A × (B | C);",
            "op g : (A → B) × C → D;",
            "op h : A × B | C → D;",
            "op i : A × B | C ⇸ D;",
            "op j : (A ⇸ B) × C → D;",
            "op k : A ⇸ B | C;",
            "var v : (A × B) × C;",
            "var w : A | (B | C);",
            "var q : Seq[A | B];",
            "lemma l x = y;\nproof\n  x;\n  = y by step;\nqed;",
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
