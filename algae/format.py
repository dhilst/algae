"""Formatter for equational .alg sources.

`respell` is what `fmt` runs: it preserves the source verbatim — whitespace,
layout, comments — and only canonicalizes symbol spellings. `Formatter` is
the AST pretty-printer, used to render synthesized types in checker errors
and available programmatically.
"""

from __future__ import annotations

from typing import Any

from .ast import (
    AliasDecl,
    EqDecl,
    IncludeDecl,
    LemmaDecl,
    LetDecl,
    Module,
    Node,
    OpDecl,
    OpenDecl,
    ParamDecl,
    PropDecl,
    RuleDecl,
    SortDecl,
)
from .parser import PRECEDENCE, WORD_SYMBOLS, lex

# Word aliases inverted, overridden where the canonical ASCII form is symbolic.
ASCII = {symbol: word for word, symbol in WORD_SYMBOLS.items()} | {
    "▷": "|>",
    "∧": "/\\",
    "∨": "\\/",
    "×": "*",
    "⇸": "-/->",
    "⊢": "|-",
}


def respell(text: str, *, ascii: bool = False) -> str:
    """Rewrite symbol tokens to their canonical spelling, byte-for-byte otherwise.

    Unicode is canonical by default; `ascii` swaps to the canonical ASCII
    aliases instead. `*` and `×` are interchangeable in both expressions
    (multiplication) and types (product), so they respell to `×`/`*` too.
    """
    line_starts = [0]
    for index, char in enumerate(text):
        if char == "\n":
            line_starts.append(index + 1)
    pieces: list[str] = []
    position = 0
    for token in lex(text):
        if token.kind != "SYMBOL":
            continue
        if ascii:
            target = ASCII.get(token.value, token.value)
        else:
            target = "×" if token.value == "*" else token.value
        if target == token.text:
            continue
        start = line_starts[token.line - 1] + token.column - 1
        pieces.append(text[position:start])
        pieces.append(target)
        position = start + len(token.text)
    pieces.append(text[position:])
    return "".join(pieces)


class Formatter:
    def __init__(self, *, ascii: bool = False) -> None:
        self.ascii = ascii

    def sym(self, value: str) -> str:
        return ASCII[value] if self.ascii and value in ASCII else value

    def format_module(self, module: Module) -> str:
        lines: list[str] = []
        previous_group = None
        for decl in module.declarations:
            group = decl.__class__.__name__
            if lines and group != previous_group:
                lines.append("")
            lines.extend(f"# {comment}".rstrip() for comment in decl.leading_comments)
            text = self.format_decl(decl)
            if decl.trailing_comment:
                text += f"  # {decl.trailing_comment}"
            lines.append(text)
            previous_group = group
        lines.extend(f"# {comment}".rstrip() for comment in module.trailing_comments)
        return "\n".join(lines) + ("\n" if lines else "")

    def format_decl(self, decl: Any) -> str:
        if isinstance(decl, SortDecl):
            return f"sort {decl.name} : {self.type_expr(decl.kind_expr)};"
        if isinstance(decl, ParamDecl):
            return f"param {decl.name} : {self.type_expr(decl.kind_expr)};"
        if isinstance(decl, OpDecl):
            arrow = self.sym("⇸" if decl.partial else "→")
            if len(decl.domain) == 1 and decl.domain[0].kind == "type_sum":
                # A lone sum argument prints unparenthesized; its branches sit
                # at product precedence so a nested arrow still gets parens
                # and cannot be mistaken for the signature arrow.
                domain = " | ".join(
                    self.type_expr(item, self.TYPE_PRODUCT) for item in decl.domain[0].data["items"]
                )
            else:
                # Domain items parse as type primaries, so looser types need parens.
                domain = f" {self.sym('×')} ".join(
                    self.type_expr(item, self.TYPE_PRIMARY) for item in decl.domain
                )
            if domain:
                return f"op {decl.name} : {domain} {arrow} {self.type_expr(decl.codomain)};"
            return f"op {decl.name} : {arrow} {self.type_expr(decl.codomain)};"
        if isinstance(decl, EqDecl):
            return self.format_decl_head(decl.expr, f"eq {self.decl_head(decl.name, decl.params)}")
        if isinstance(decl, PropDecl):
            return self.format_decl_head(decl.expr, f"prop {self.decl_head(decl.name, decl.params)}")
        if isinstance(decl, LemmaDecl):
            lines = [self.format_decl_head(decl.expr, f"lemma {self.decl_head(decl.name, decl.params)}")]
            if decl.proof is not None:
                lines.append("proof")
                lines.extend(self.format_proof_steps(decl.proof.data["steps"], "  "))
                lines.append(f"{decl.proof.data['terminator']};")
            return "\n".join(lines)
        if isinstance(decl, RuleDecl):
            lines = [f"rule {decl.name}{self.binder_list(decl.params)}"]
            for premise in decl.premises:
                lines.append(f"  case {premise.data['name']}")
                lines.append(f"    {self.prop(premise.data['prop'])}")
                lines.append("  end;")
            lines.append("  ─────────────────────")
            lines.append(f"  {self.prop(decl.conclusion)}")
            lines.append("end;")
            return "\n".join(lines)
        if isinstance(decl, LetDecl):
            return f"let {decl.name} = {self.expr(decl.expr)};"
        if isinstance(decl, IncludeDecl):
            head = f"include {'::'.join(decl.path)}"
            if decl.bindings:
                bindings = ", ".join(f"{name} := {self.type_expr(t)}" for name, t in decl.bindings)
                head += f" with ({bindings})"
            if decl.obligations:
                lines = [head + " props"]
                for case in decl.obligations:
                    lines.append(f"  case {case.data['name']}")
                    lines.extend(self.format_proof_steps(case.data["steps"], "    "))
                    lines.append(f"  {case.data['terminator']};")
                lines.append(f"{decl.obligations_terminator};")
                return "\n".join(lines)
            return head + ";"
        if isinstance(decl, OpenDecl):
            return f"open {'::'.join(decl.path)} ({', '.join(decl.names)});"
        if isinstance(decl, AliasDecl):
            return f"alias {decl.alias} = {'::'.join(decl.path)};"
        raise TypeError(f"unsupported declaration: {decl!r}")

    def format_proof_steps(self, steps: list[Any], indent: str) -> list[str]:
        lines: list[str] = []
        for step in steps:
            tactic = step.data["tactic"]
            result = step.data["result"]
            lines.append(f"{indent}goal")
            lines.append(f"{indent}  {self.prop(step.data['goal'])}")
            if tactic.kind == "apply":
                # cases follow `by apply …`; the subproof terminator follows
                # `therefore <result>`.
                args = ", ".join(self.expr(arg) for arg in tactic.data["args"])
                lines.append(f"{indent}by apply {tactic.data['rule']}({args})")
                for case in tactic.data["cases"]:
                    lines.append(f"{indent}  case {case.data['name']}")
                    lines.extend(self.format_proof_steps(case.data["steps"], indent + "    "))
                    lines.append(f"{indent}  {case.data['terminator']};")
                if isinstance(result, Node) and result.kind == "done":
                    lines.append(f"{indent}therefore done")
                else:
                    lines.append(f"{indent}therefore")
                    lines.append(f"{indent}  {self.prop(result)}")
                lines.append(f"{indent}{tactic.data['terminator']};")
            else:
                tactic_lines = self.format_tactic_lines(tactic)
                lines.append(f"{indent}by {tactic_lines[0]}")
                lines.extend(f"{indent}{line}" for line in tactic_lines[1:])
                if isinstance(result, Node) and result.kind == "done":
                    lines.append(f"{indent}therefore done;")
                else:
                    lines.append(f"{indent}therefore")
                    lines.append(f"{indent}  {self.prop(result)};")
        return lines

    def format_tactic_lines(self, tactic: Node) -> list[str]:
        # Returns lines whose first element is meant to follow `by `. Apply is
        # rendered by format_proof_steps (its cases/terminator straddle the step).
        if tactic.kind == "wip_tactic":
            return ["wip"]
        if tactic.kind == "rewrite":
            thm = self.format_theorem(tactic.data["theorem"])
            lhs = self.expr(tactic.data["lhs"])
            rhs = self.expr(tactic.data["rhs"])
            return [f"rewrite {tactic.data['direction']} {thm} with ({lhs} := {rhs})"]
        raise TypeError(f"unsupported tactic: {tactic!r}")

    def format_theorem(self, theorem: Node) -> str:
        name = theorem.data["name"]
        if theorem.data.get("applied"):
            return f"{name}(" + ", ".join(self.expr(arg) for arg in theorem.data["args"]) + ")"
        return name

    def decl_head(self, name: str, params: list[Any]) -> str:
        return name if not params else f"{name}{self.binder_list(params)}"

    def format_decl_head(self, expr: Any, keyword: str) -> str:
        if isinstance(expr, Node) and expr.kind in ("let", "let_tuple"):
            return self.format_decl_lets(expr, keyword)
        return f"{keyword} {self.prop(expr)};"

    def prop(self, value: Any) -> str:
        if isinstance(value, Node) and value.kind == "sequent":
            goal = self.expr(value.data["goal"])
            assumptions = ", ".join(self.assumption(a) for a in value.data["assumptions"])
            turnstile = self.sym("⊢")
            return f"{assumptions} {turnstile} {goal}" if assumptions else f"{turnstile} {goal}"
        return self.expr(value)

    def assumption(self, value: Node) -> str:
        if value.kind == "context_var":
            return f"{value.data['name']} : {self.type_expr(value.data['type'])}"
        rendered = self.expr(value.data["expr"])
        name = value.data["name"]
        return f"{name} := {rendered}" if name is not None else rendered

    def format_decl_lets(self, expr: Node, keyword: str) -> str:
        # A let chain at the declaration spine breaks after each `in`, with the
        # bindings and final body aligned under the first one.
        lines: list[str] = []
        current: Any = expr
        while isinstance(current, Node) and current.kind in ("let", "let_tuple"):
            lines.append(f"{self.let_binding(current)} in")
            current = current.data["body"]
        lines.append(f"{self.expr(current)};")
        indent = " " * len(f"{keyword} ")
        return f"{keyword} " + f"\n{indent}".join(lines)

    # Type grammar precedence: sum `|` (1) < arrow `→` (2, right-assoc)
    # < product `×` (3) < primary (4). Children that bind looser than the
    # surrounding context get parenthesized so formatting preserves the AST.
    TYPE_SUM, TYPE_ARROW, TYPE_PRODUCT, TYPE_PRIMARY = 1, 2, 3, 4

    def type_expr(self, value: Any, min_prec: int = 0) -> str:
        if not isinstance(value, Node):
            raise TypeError(f"unsupported type expression: {value!r}")
        data = value.data
        if value.kind == "type_name":
            qualified = "::".join([*data.get("module", []), data["name"]])
            args = data.get("args", [])
            if args:
                return qualified + "[" + ", ".join(self.type_expr(arg) for arg in args) + "]"
            return qualified
        if value.kind == "type_builtin":
            return self.sym(data["name"])
        if value.kind == "type_unit":
            return "()"
        if value.kind == "type_sequence":
            return f"Seq[{self.type_expr(data['item'])}]"
        if value.kind == "type_function":
            left = self.type_expr(data["left"], self.TYPE_PRODUCT)
            right = self.type_expr(data["right"], self.TYPE_ARROW)
            # `.get`: nodes synthesized by the checker do not set the key.
            arrow = self.sym("⇸" if data.get("partial") else "→")
            return self.wrap(f"{left} {arrow} {right}", self.TYPE_ARROW, min_prec)
        if value.kind == "type_product":
            text = f" {self.sym('×')} ".join(
                self.type_expr(item, self.TYPE_PRIMARY) for item in data["items"]
            )
            return self.wrap(text, self.TYPE_PRODUCT, min_prec)
        if value.kind == "type_sum":
            text = " | ".join(self.type_expr(item, self.TYPE_ARROW) for item in data["items"])
            return self.wrap(text, self.TYPE_SUM, min_prec)
        raise TypeError(f"unsupported type expression: {value!r}")

    @staticmethod
    def wrap(text: str, prec: int, min_prec: int) -> str:
        return f"({text})" if prec < min_prec else text

    # Expression precedence beyond the parser's binary table: unary operands
    # parse at level 9, postfix (call, prime) binds tighter, and atoms never
    # need parens. if/let extend greedily to the right, so they sit at the
    # bottom and get parenthesized in any operand position.
    UNARY_PREC = 8
    POSTFIX_PREC = 10
    ATOM_PREC = 11

    def expr(self, value: Any, min_prec: int = 0) -> str:
        if not isinstance(value, Node):
            raise TypeError(f"unsupported expression: {value!r}")
        data = value.data
        if value.kind == "identifier":
            return data["name"]
        if value.kind == "qualified":
            return "::".join([*data["module"], data["name"]])
        if value.kind == "bool":
            return "true" if data["value"] else "false"
        if value.kind == "bool_symbol":
            return self.sym(data["value"])
        if value.kind == "builtin_set":
            return self.sym(data["name"])
        if value.kind == "unit":
            return "()"
        if value.kind == "tuple":
            return "(" + ", ".join(self.expr(item) for item in data["items"]) + ")"
        if value.kind == "prime":
            return self.expr(data["value"], self.POSTFIX_PREC) + "'"
        if value.kind == "call":
            function = self.expr(data["function"], self.POSTFIX_PREC)
            return function + "(" + ", ".join(self.expr(arg) for arg in data["args"]) + ")"
        if value.kind == "unary":
            text = self.sym(data["op"]) + " " + self.expr(data["value"], PRECEDENCE["."])
            return self.wrap(text, self.UNARY_PREC, min_prec)
        if value.kind == "binary":
            op = data["op"]
            prec = PRECEDENCE[op]
            right_assoc = op in {"⟹", "⟺"}
            left = self.expr(data["left"], prec + 1 if right_assoc else prec)
            right = self.expr(data["right"], prec if right_assoc else prec + 1)
            text = f"{left}.{right}" if op == "." else f"{left} {self.sym(op)} {right}"
            return self.wrap(text, prec, min_prec)
        if value.kind == "if":
            text = (
                f"if {self.expr(data['condition'])} then {self.expr(data['then'])} "
                f"else {self.expr(data['otherwise'])}"
            )
            return self.wrap(text, 0, min_prec)
        if value.kind in ("let", "let_tuple"):
            text = f"{self.let_binding(value)} in {self.expr(data['body'])}"
            return self.wrap(text, 0, min_prec)
        if value.kind == "lambda":
            text = f"{self.sym('λ')} {self.binder_list(data['binders'])} => {self.expr(data['body'])}"
            return self.wrap(text, 0, min_prec)
        if value.kind in ("forall", "exists"):
            quantifier = self.sym("∀" if value.kind == "forall" else "∃")
            text = f"{quantifier} {self.binder_list(data['binders'])} st {self.expr(data['body'])}"
            return self.wrap(text, 0, min_prec)
        raise TypeError(f"unsupported expression: {value!r}")

    def binder_list(self, binders: list[Any]) -> str:
        # Group consecutive binders that share a type back into `a b : T` and
        # join entries with commas: (a : A, b c : B).
        entries: list[str] = []
        index = 0
        while index < len(binders):
            rendered = self.type_expr(binders[index][1])
            names = [binders[index][0]]
            index += 1
            while index < len(binders) and self.type_expr(binders[index][1]) == rendered:
                names.append(binders[index][0])
                index += 1
            entries.append(f"{' '.join(names)} : {rendered}")
        return "(" + ", ".join(entries) + ")"

    def let_binding(self, value: Node) -> str:
        if value.kind == "let_tuple":
            pattern = "(" + ", ".join(value.data["binders"]) + ")"
        else:
            pattern = value.data["name"]
        return f"let {pattern} = {self.expr(value.data['value'])}"


def format_spec(module: Module, *, ascii: bool = False) -> str:
    return Formatter(ascii=ascii).format_module(module)
