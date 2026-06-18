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

from algae.ast import EqDecl, LemmaDecl, LetDecl, OpDecl, ParamDecl, PropDecl, SortDecl  # noqa: E402
from algae.format import format_spec  # noqa: E402
from algae.parser import parse_text  # noqa: E402

# A self-contained naturals module: a user-declared Nat sort with z/successor,
# a base equation, and the induction rule with named premises. Tests append
# lemmas and proofs to this.
NAT_PREAMBLE = "\n".join(
    [
        "sort Nat : Sort;",
        "op z : → Nat;",
        "op s : Nat → Nat;",
        "op add : Nat × Nat → Nat;",
        "eq add_zero_left(n : Nat) add(z, n) = n;",
        "eq add_succ_left(n m : Nat) add(s(n), m) = s(add(n, m));",
        "rule reflexivity(T : Sort, x : T)",
        "  ─────",
        "  ⊢ x = x",
        "end;",
        "rule induction(P : Nat → Prop)",
        "  case base",
        "    ⊢ P(z)",
        "  end;",
        "  case step",
        "    n : Nat, P(n) ⊢ P(s(n))",
        "  end;",
        "  ─────",
        "  ⊢ ∀ (n : Nat) st P(n)",
        "end;",
        "",
    ]
)


def semantic_payload(module) -> list[tuple]:
    # The declaration payloads that formatting must preserve, ignoring line
    # numbers and comments. Nodes are dataclasses, so == is structural.
    payload: list[tuple] = []
    for decl in module.declarations:
        if isinstance(decl, SortDecl):
            payload.append(("sort", decl.name, decl.kind_expr))
        elif isinstance(decl, ParamDecl):
            payload.append(("param", decl.name, decl.kind_expr))
        elif isinstance(decl, OpDecl):
            payload.append(("op", decl.name, decl.domain, decl.codomain, decl.partial))
        elif isinstance(decl, EqDecl):
            payload.append(("eq", decl.name, decl.expr, decl.params))
        elif isinstance(decl, PropDecl):
            payload.append(("prop", decl.name, decl.expr, decl.params))
        elif isinstance(decl, LemmaDecl):
            payload.append(("lemma", decl.name, decl.expr, decl.proof, decl.params))
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

    def check_in_project(self, source: str, where: str = "examples/proj") -> subprocess.CompletedProcess[str]:
        path = ROOT / where / "_tmp_spec.alg"
        path.write_text(source, encoding="utf-8")
        try:
            return self.run_cli("check", str(path.relative_to(ROOT)))
        finally:
            path.unlink(missing_ok=True)

    # Acceptance and rejection corpora ---------------------------------------

    def test_check_accepts_core_fixtures(self) -> None:
        result = self.run_cli(
            "check", "examples/stack.alg", "examples/kvstore.alg", "examples/base/container.alg"
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("examples/stack.alg: ok", result.stdout)
        self.assertIn("examples/kvstore.alg: ok", result.stdout)
        self.assertIn("examples/base/container.alg: ok", result.stdout)

    def test_all_examples_check(self) -> None:
        # Every example spec must type-check — standalone specs and project
        # specs alike (the latter resolve includes via their nearest
        # alg-project.json). Guards the examples/ tree against rot.
        paths = sorted(str(p.relative_to(ROOT)) for p in (ROOT / "examples").rglob("*.alg"))
        self.assertTrue(paths, "no example .alg files found")
        result = self.run_cli("check", *paths)
        self.assertEqual(result.returncode, 0, result.stdout)
        for path in paths:
            self.assertIn(f"{path}: ok", result.stdout)

    def test_check_rejects_old_and_malformed_syntax(self) -> None:
        paths = sorted(str(path.relative_to(ROOT)) for path in (ROOT / "tests/reject").glob("*.alg"))
        result = self.run_cli("check", *paths)
        self.assertEqual(result.returncode, 1)
        self.assertIn("Expected", result.stdout)
        self.assertIn("found", result.stdout)
        self.assertNotIn(": ok", result.stdout)

    def test_canonical_nat_and_monoid_fixtures(self) -> None:
        # The two canonical new-syntax fixtures: the full induction proof and
        # the module obligation discharge.
        result = self.run_cli(
            "check", "examples/nat-with-induction.alg", "examples/monoid/nat_monoid.alg"
        )
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertIn("examples/nat-with-induction.alg: ok", result.stdout)
        self.assertIn("examples/monoid/nat_monoid.alg: ok", result.stdout)

    # Print / AST ------------------------------------------------------------

    def test_print_outputs_json_ast(self) -> None:
        result = self.run_cli("print", "examples/stack.alg")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["ast"]["kind"], "Module")
        kinds = [d["kind"] for d in payload["ast"]["declarations"]]
        self.assertEqual(kinds[0], "SortDecl")
        self.assertIn("OpDecl", kinds)
        self.assertIn("EqDecl", kinds)

    # Formatting -------------------------------------------------------------

    def test_fmt_converts_ascii_aliases_to_unicode(self) -> None:
        source = "\n".join(
            [
                "sort Stack : Sort;",
                "sort Elem : Sort;",
                "op empty : arrow Stack;",
                "op push : Stack product Elem arrow Stack;",
                "eq pop_push (s : Stack, e : Elem) push(s, e) neq empty();",
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
        self.assertIn("push(s, e) ≠ empty()", result.stdout)

    def test_fmt_ascii_outputs_keyword_aliases(self) -> None:
        result = self.run_cli("fmt", "--ascii", "examples/stack.alg")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Stack * Elem arrow Stack", result.stdout)
        self.assertNotIn("→", result.stdout)
        self.assertNotIn("×", result.stdout)

    def test_fmt_preserves_whitespace_and_layout(self) -> None:
        source = "\n".join(
            [
                "sort Elem : Sort;",
                "",
                "op  q  :  arrow   Elem;",
                "eq  e_eq  q   =    q;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "layout.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", str(path))
        self.assertEqual(result.returncode, 0, result.stderr)
        # Only the alias `arrow` is respelled; spacing and layout stay verbatim.
        self.assertEqual(result.stdout, source.replace("arrow", "→"))

    def test_fmt_preserves_comments(self) -> None:
        source = "\n".join(
            [
                "# A simple stack.",
                "sort Stack : Sort;",
                "op empty : arrow Stack;  # constructor",
                "eq empty_eq empty() = empty();",
                "# end of spec",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "commented.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", str(path))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("# A simple stack.\nsort Stack : Sort;", result.stdout)
        self.assertIn("op empty : → Stack;  # constructor", result.stdout)
        self.assertIn("# end of spec", result.stdout)

    def test_fmt_inplace_respells_but_preserves_layout(self) -> None:
        source = "sort Stack:Sort;op empty:arrow Stack;eq empty_eq empty()=empty();\n"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "stack.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", "--inplace", str(path))
            rewritten = path.read_text(encoding="utf-8")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertEqual(
            rewritten, "sort Stack:Sort;op empty:→ Stack;eq empty_eq empty()=empty();\n"
        )

    # Sorts, params, and kinds -----------------------------------------------

    def test_sort_kind_declarations_check(self) -> None:
        source = "\n".join(
            [
                "sort Nat : Sort;",
                "sort List : Sort → Sort;",
                "sort Pair : Sort → Sort → Sort;",
                "op nil : → List[Nat];",
                "op mk : Nat × Nat → Pair[Nat, Nat];",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_sort_arity_errors(self) -> None:
        self.assertIn(
            "sort List takes 1 type argument(s), got 0",
            self.check_source("sort List : Sort → Sort;\nop bad : → List;\n").stdout,
        )
        self.assertIn(
            "sort Pair takes 2 type argument(s), got 1",
            self.check_source(
                "sort A : Sort;\nsort Pair : Sort → Sort → Sort;\nop bad : → Pair[A];\n"
            ).stdout,
        )
        self.assertIn(
            "unknown sort Nope",
            self.check_source("sort A : Sort;\nop bad : → Nope;\n").stdout,
        )

    def test_kind_must_be_sort(self) -> None:
        result = self.check_source("sort Bad : Foo;\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("a kind must be Sort", result.stdout)

    def test_param_declaration_checks(self) -> None:
        source = "\n".join(
            [
                "param T : Sort;",
                "sort List : Sort → Sort;",
                "op nil : → List[T];",
                "op cons : T × List[T] → List[T];",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_duplicate_sort_rejected(self) -> None:
        result = self.check_source("sort S : Sort;\nsort S : Sort;\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("duplicate sort S", result.stdout)

    # eq / prop / lemma ------------------------------------------------------

    def test_eq_binders_and_nullary_constant(self) -> None:
        source = "\n".join(
            [
                "sort Nat : Sort;",
                "op z : → Nat;",
                "op add : Nat × Nat → Nat;",
                "eq add_zero_left(n : Nat) add(z, n) = n;",  # z used bare as a constant
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_eq_equate_type_mismatch(self) -> None:
        source = "\n".join(
            [
                "sort A : Sort;",
                "sort B : Sort;",
                "op a : → A;",
                "op b : → B;",
                "eq bad a = b;",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("cannot equate A with B", result.stdout)

    def test_duplicate_eq_name_rejected(self) -> None:
        source = "sort S : Sort;\nop a : → S;\neq dup a = a;\neq dup a = a;\n"
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("duplicate eq name dup", result.stdout)

    def test_prop_declaration_checks(self) -> None:
        source = "\n".join(
            [
                "param T : Sort;",
                "op unit : → T;",
                "op mul : T × T → T;",
                "prop left_identity(x : T) mul(unit, x) = x;",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_lemma_without_proof_checks(self) -> None:
        source = "sort S : Sort;\nop f : S → S;\nlemma f_id(x : S) f(f(x)) = f(x);\n"
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_lemma_proof_rewrite_steps_not_discharged(self) -> None:
        # The lemma proposition is checked; the proof's rewrite steps are
        # recorded structurally and never discharged.
        source = "\n".join(
            [
                "sort S : Sort;",
                "op a : → S;",
                "op b : → S;",
                "eq ab a = b;",
                "lemma plausible a = b;",
                "proof",
                "  goal",
                "    ⊢ a = b",
                "  by rewrite > ab with (a := b)",
                "  therefore",
                "    ⊢ b = b;",
                "qed;",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_builtin_in_term_position_rejected(self) -> None:
        result = self.check_source("eq bad 𝔹 = 𝔹;\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("𝔹 is a sort, not a term", result.stdout)

    def test_undeclared_identifier_rejected(self) -> None:
        result = self.check_source("sort S : Sort;\nop a : → S;\neq bad a = missing;\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("undeclared identifier missing", result.stdout)

    # Rules ------------------------------------------------------------------

    def test_rule_named_premises_and_typed_context(self) -> None:
        result = self.check_source(NAT_PREAMBLE)
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_rule_premise_rejects_named_assumption(self) -> None:
        source = "\n".join(
            [
                "rule bad(A : Prop, B : Prop)",
                "  case c",
                "    h := A ⊢ B",
                "  end;",
                "  ─────",
                "  ⊢ A",
                "end;",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("rule premise assumptions must be unnamed", result.stdout)

    def test_rejects_non_proposition_goal(self) -> None:
        source = "sort S : Sort;\nop a : → S;\nrule r()\n  case c\n    ⊢ a\n  end;\n  ─────\n  ⊢ a\nend;\n"
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("goal must be a proposition, got S", result.stdout)

    def test_duplicate_name_across_eq_and_rule(self) -> None:
        source = NAT_PREAMBLE + "eq induction(n : Nat) add(z, n) = n;\n"
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("duplicate eq name induction", result.stdout)

    # Proofs: goal / by / therefore / done, apply, rewrite -------------------

    def lemma_with_proof(self, body: str) -> str:
        return NAT_PREAMBLE + "\n".join(
            ["lemma add_zero_right(n : Nat) add(n, z) = n;", "proof", body, "qed;", ""]
        )

    def test_apply_rejects_arg_count_mismatch(self) -> None:
        source = self.lemma_with_proof(
            "  goal\n    ⊢ add(n, z) = n\n  by apply induction()\n  therefore done qed;"
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("rule induction expects 1 argument(s), got 0", result.stdout)

    def test_apply_rejects_case_name_mismatch(self) -> None:
        source = self.lemma_with_proof(
            "\n".join(
                [
                    "  goal",
                    "    ⊢ add(n, z) = n",
                    "  by apply induction(λ (n : Nat) => add(n, z) = n)",
                    "    case base",
                    "    qed;",
                    "  therefore done",
                    "  qed;",
                ]
            )
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("requires cases ['base', 'step'], got ['base']", result.stdout)

    def test_apply_rejects_unknown_rule(self) -> None:
        source = self.lemma_with_proof(
            "  goal\n    ⊢ add(n, z) = n\n  by apply nonexistent(z)\n  therefore done qed;"
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("unknown rule nonexistent", result.stdout)

    def test_apply_rejects_non_predicate_argument(self) -> None:
        # Passing a bare Nat where the induction predicate Nat → Prop is required.
        source = self.lemma_with_proof(
            "\n".join(
                [
                    "  goal",
                    "    ⊢ add(n, z) = n",
                    "  by apply induction(z)",
                    "    case base",
                    "    qed;",
                    "    case step",
                    "    qed;",
                    "  therefore done",
                    "  qed;",
                ]
            )
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("argument for P has type Nat, expected Nat → Prop", result.stdout)

    def test_rule_application_ast_shapes(self) -> None:
        result = self.run_cli("print", "examples/nat-with-induction.alg")
        self.assertEqual(result.returncode, 0, result.stderr)
        declarations = json.loads(result.stdout)["ast"]["declarations"]
        rule = next(d for d in declarations if d["kind"] == "RuleDecl" and d["name"] == "induction")
        self.assertEqual([p["data"]["name"] for p in rule["premises"]], ["base", "step"])
        self.assertEqual(rule["conclusion"]["data"]["goal"]["kind"], "forall")
        lemma = next(d for d in declarations if d["kind"] == "LemmaDecl")
        step = lemma["proof"]["data"]["steps"][0]
        self.assertEqual(step["kind"], "proof_step")
        self.assertEqual(step["data"]["tactic"]["data"]["rule"], "induction")
        self.assertEqual(step["data"]["result"]["kind"], "done")

    def test_zero_premise_apply_uses_qed(self) -> None:
        source = self.lemma_with_proof(
            "  goal\n    ⊢ add(n, z) = n\n  by apply reflexivity(Nat, n)\n  therefore done qed;"
        )
        self.assertEqual(self.check_source(source).returncode, 0, self.check_source(source).stdout)

    def test_wip_tactic_closed_with_wip(self) -> None:
        # A proof that is still work in progress is closed with `wip`, not `qed`.
        source = NAT_PREAMBLE + "\n".join(
            [
                "lemma todo(n : Nat) add(n, z) = n;",
                "proof",
                "  goal",
                "    ⊢ add(n, z) = n",
                "  by wip",
                "  therefore",
                "    ⊢ add(n, z) = n;",
                "wip;",
                "",
            ]
        )
        self.assertEqual(self.check_source(source).returncode, 0, self.check_source(source).stdout)

    def test_wip_with_qed_is_rejected(self) -> None:
        source = self.lemma_with_proof(
            "  goal\n    ⊢ add(n, z) = n\n  by wip\n  therefore\n    ⊢ add(n, z) = n;"
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("close it with `wip`, not `qed`", result.stdout)

    def test_wip_is_viral_through_apply(self) -> None:
        # A case left `wip` makes the enclosing apply and proof work in progress,
        # which a `qed` on the apply rejects.
        source = self.lemma_with_proof(
            "\n".join(
                [
                    "  goal",
                    "    ⊢ add(n, z) = n",
                    "  by apply induction(λ (n : Nat) => add(n, z) = n)",
                    "    case base",
                    "      goal",
                    "        ⊢ add(z, z) = z",
                    "      by wip",
                    "      therefore",
                    "        ⊢ add(z, z) = z;",
                    "    wip;",
                    "    case step",
                    "    qed;",
                    "  therefore done",
                    "  qed;",
                ]
            )
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("apply induction is work in progress", result.stdout)

    # Modules ----------------------------------------------------------------

    def test_module_include_open_alias_ok(self) -> None:
        result = self.run_cli("check", "examples/proj/use_list.alg")
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertIn("examples/proj/use_list.alg: ok", result.stdout)

    def test_module_print_shapes(self) -> None:
        result = self.run_cli("print", "examples/proj/use_list.alg")
        self.assertEqual(result.returncode, 0, result.stderr)
        kinds = [d["kind"] for d in json.loads(result.stdout)["ast"]["declarations"]]
        self.assertEqual(kinds[:3], ["IncludeDecl", "AliasDecl", "OpenDecl"])

    def test_module_open_requires_include(self) -> None:
        result = self.check_in_project("open list (nil);\nsort Elem : Sort;\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("open of un-included module list", result.stdout)

    def test_module_missing_module(self) -> None:
        result = self.check_in_project("include nope::missing;\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("module nope::missing not found", result.stdout)

    def test_include_without_project_rejected(self) -> None:
        result = self.check_source("include list;\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("alg-project.json", result.stdout)

    def test_include_without_with_keeps_param_abstract(self) -> None:
        # No `with`: the parameter stays abstract. The namespaced sort is still
        # usable; instantiating cons at a concrete element type is not (it would
        # require binding T), which is the point of leaving it abstract.
        result = self.check_in_project(
            "include list;\nsort Elem : Sort;\n"
            "eq a(xs : list::List[Elem]) xs = xs;\n"
        )
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_module_open_unexported_name(self) -> None:
        result = self.check_in_project(
            "include list with (T := Elem);\nopen list (bogus);\nsort Elem : Sort;\n"
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("not exported by module list", result.stdout)

    def test_include_obligation_missing_case_rejected(self) -> None:
        source = "\n".join(
            [
                "include nat;",
                "open nat (z, s, add);",
                "include monoid with (T := nat::Nat, unit := z, mul := add) props",
                "  case left_identity",
                "    goal",
                "      ⊢ add(z, x) = x",
                "    by rewrite > add_zero_left(x) with (add(z, x) := x)",
                "    therefore done;",
                "  qed;",
                "qed;",
                "",
            ]
        )
        result = self.check_in_project(source, where="examples/monoid")
        self.assertEqual(result.returncode, 1)
        self.assertIn("requires obligation cases", result.stdout)

    def test_include_obligations_wip_is_viral(self) -> None:
        # A `wip` obligation case makes the whole `props` block work in progress;
        # closing it with `qed` is rejected.
        source = "\n".join(
            [
                "include nat;",
                "open nat (z, s, add);",
                "include monoid with (T := nat::Nat, unit := z, mul := add) props",
                "  case left_identity",
                "    goal",
                "      ⊢ add(z, x) = x",
                "    by wip",
                "    therefore",
                "      ⊢ add(z, x) = x;",
                "  wip;",
                "  case associativity",
                "  qed;",
                "qed;",
                "",
            ]
        )
        result = self.check_in_project(source, where="examples/monoid")
        self.assertEqual(result.returncode, 1)
        self.assertIn("include monoid is work in progress", result.stdout)

    # Expression features ----------------------------------------------------

    def test_quantifier_and_lambda(self) -> None:
        source = "\n".join(
            [
                "sort A : Sort;",
                "sort B : Sort;",
                "op f : A × B → A;",
                "eq a(x : A) (∀ (u : A, v : B) st f(u, v) = u) ∧ x = x;",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_application_sugar(self) -> None:
        source = "\n".join(
            [
                "sort S : Sort;",
                "op f : S × S → S;",
                "eq sugar_first(x : S) x.f(x).f(x) = f(f(x, x), x);",
                "eq sugar_last(x : S) x ▷ f(x) = f(x, x);",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)
        module = parse_text(source)
        self.assertIn("x ▷ f(x) = f(x, x)", format_spec(module))
        self.assertIn("x |> f(x) = f(x, x)", format_spec(module, ascii=True))

    def test_destructuring_let(self) -> None:
        source = "\n".join(
            [
                "sort S : Sort;",
                "sort T : Sort;",
                "op pair : S → S × T;",
                "eq pair_fst(x : S) let (a, _) = pair(x) in a = x;",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_destructuring_duplicate_binder_rejected(self) -> None:
        source = "\n".join(
            [
                "sort S : Sort;",
                "sort T : Sort;",
                "op pair : S × T → S × T;",
                "eq dup(s : S, t : T) let (x, x) = pair(s, t) in x = t;",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("duplicate binder x in destructuring pattern", result.stdout)

    def test_partial_op_and_sum_domain(self) -> None:
        source = "\n".join(
            [
                "sort S : Sort;",
                "sort Error : Sort;",
                "op oops : → Error;",
                "op f : → S | Error;",
                "op assert : S | Error ⇸ S;",
                "op coerce : S -/-> S;",
                "eq assert_elim(x : S) f().assert = x;",
                "",
            ]
        )
        result = self.check_source(source)
        self.assertEqual(result.returncode, 0, result.stdout)
        module = parse_text(source)
        partial = {d.name: d.partial for d in module.declarations if isinstance(d, OpDecl)}
        self.assertEqual(partial, {"oops": False, "f": False, "assert": True, "coerce": True})
        self.assertIn("op assert : S | Error -/-> S;", format_spec(module, ascii=True))

    def test_implicit_narrowing_rejected_but_cast_accepted(self) -> None:
        bad = "\n".join(
            [
                "sort S : Sort;",
                "sort Error : Sort;",
                "op oops : → Error;",
                "op f : → S | Error;",
                "op g : S → S;",
                "op a : → S;",
                "eq narrow g(f()) = a;",
                "",
            ]
        )
        self.assertIn("no signature of g matches (S | Error)", self.check_source(bad).stdout)
        good = bad.replace("eq narrow", "op cast : (S | Error) → S;\neq narrow").replace(
            "g(f())", "g(cast(f()))"
        )
        self.assertEqual(self.check_source(good).returncode, 0, self.check_source(good).stdout)

    def test_sequence_concat(self) -> None:
        source = "\n".join(
            [
                "sort Elem : Sort;",
                "op a : → Seq[Elem];",
                "eq concat a() ++ a() = a();",
                "",
            ]
        )
        self.assertEqual(self.check_source(source).returncode, 0)
        bad = "op a : → Seq[Elem];\nop b : → Seq[Elem];\nsort Elem : Sort;\nsort Other : Sort;\n"
        bad += "op c : → Seq[Other];\neq x a() ++ c() = a();\n"
        self.assertIn("++ requires matching Seq operands", self.check_source(bad).stdout)

    # Formatting round-trips -------------------------------------------------

    def test_format_spec_round_trips(self) -> None:
        sources = [
            "sort Nat : Sort;",
            "sort List : Sort → Sort;",
            "param T : Sort;",
            "op f : → A × (B | C);",
            "op g : (A → B) × C → D;",
            "op h : A × B | C → D;",
            "op i : A × B | C ⇸ D;",
            "eq r(a b c d : N) (a.f).g = d;",
            "eq pipe(a b : N) a ▷ f(b) = c;",
            "prop p(x : T) f(x) = x;",
            "lemma l(x : S) x = x;",
        ]
        for source in sources:
            with self.subTest(source=source):
                module = parse_text(source)
                rendered = format_spec(module)
                reparsed = parse_text(rendered)
                self.assertEqual(semantic_payload(reparsed), semantic_payload(module), rendered)
                self.assertEqual(format_spec(reparsed), rendered)

    def test_format_spec_round_trips_full_proof(self) -> None:
        # The AST pretty-printer renders the full induction proof to a form that
        # re-parses and re-renders identically (stable through parse → format).
        module = parse_text((ROOT / "examples/nat-with-induction.alg").read_text(encoding="utf-8"))
        rendered = format_spec(module)
        reparsed = parse_text(rendered)
        self.assertEqual(format_spec(reparsed), rendered)
        self.assertEqual(semantic_payload(reparsed), semantic_payload(module))


if __name__ == "__main__":
    unittest.main()
