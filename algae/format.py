"""Formatter for equational .alg ASTs."""

from __future__ import annotations

from typing import Any

from .ast import AxiomDecl, LetDecl, Module, Node, OpDecl, SortDecl, VarDecl
from .parser import PRECEDENCE, WORD_SYMBOLS

ASCII = {symbol: word for word, symbol in WORD_SYMBOLS.items()} | {"▷": "|>"}


class Formatter:
    def __init__(self, *, ascii: bool = False, valign: bool = True) -> None:
        self.ascii = ascii
        self.valign = valign

    def sym(self, value: str) -> str:
        return ASCII[value] if self.ascii and value in ASCII else value

    def format_module(self, module: Module) -> str:
        lines: list[str] = []
        widths = self.name_widths(module.declarations)
        previous_group = None
        for decl, width in zip(module.declarations, widths):
            group = decl.__class__.__name__
            if lines and group != previous_group:
                lines.append("")
            lines.extend(f"# {comment}".rstrip() for comment in decl.leading_comments)
            text = self.format_decl(decl, name_width=width)
            if decl.trailing_comment:
                text += f"  # {decl.trailing_comment}"
            lines.append(text)
            previous_group = group
        lines.extend(f"# {comment}".rstrip() for comment in module.trailing_comments)
        return "\n".join(lines) + ("\n" if lines else "")

    def name_widths(self, declarations: list[Any]) -> list[int]:
        # Pad within each run of same-kind declarations so the `:` (op/var)
        # and `=` (let, single-line `=` axioms) separators align vertically.
        # A leading comment starts a new run, so commented subgroups align
        # independently.
        widths = [0] * len(declarations)
        if not self.valign:
            return widths
        index = 0
        while index < len(declarations):
            kind = declarations[index].__class__
            end = index + 1
            while (
                end < len(declarations)
                and declarations[end].__class__ is kind
                and not declarations[end].leading_comments
            ):
                end += 1
            run = declarations[index:end]
            if kind in (OpDecl, VarDecl, LetDecl):
                width = max(len(decl.name) for decl in run)
                for position in range(index, end):
                    widths[position] = width
            elif kind is AxiomDecl:
                lengths = [
                    len(self.expr(decl.expr.data["left"], PRECEDENCE["="]))
                    for decl in run
                    if self.axiom_aligns(decl)
                ]
                if lengths:
                    width = max(lengths)
                    for position in range(index, end):
                        if self.axiom_aligns(declarations[position]):
                            widths[position] = width
            index = end
        return widths

    def axiom_aligns(self, decl: AxiomDecl) -> bool:
        # Only single-line axioms whose top-level operator is `=` take part
        # in alignment; let-chain axioms format multiline.
        expr = decl.expr
        return isinstance(expr, Node) and expr.kind == "binary" and expr.data["op"] == "="

    def format_decl(self, decl: Any, name_width: int = 0) -> str:
        if isinstance(decl, SortDecl):
            if decl.values is not None:
                return f"sort {decl.names[0]} = " + "{" + ", ".join(decl.values) + "};"
            return f"sort {', '.join(decl.names)};"
        if isinstance(decl, OpDecl):
            name = decl.name.ljust(name_width)
            # Domain items parse as type primaries, so looser types need parens.
            domain = f" {self.sym('×')} ".join(
                self.type_expr(item, self.TYPE_PRIMARY) for item in decl.domain
            )
            if domain:
                return f"op {name} : {domain} {self.sym('→')} {self.type_expr(decl.codomain)};"
            return f"op {name} : {self.sym('→')} {self.type_expr(decl.codomain)};"
        if isinstance(decl, VarDecl):
            return f"var {decl.name.ljust(name_width)} : {self.type_expr(decl.sort)};"
        if isinstance(decl, AxiomDecl):
            if isinstance(decl.expr, Node) and decl.expr.kind in ("let", "let_tuple"):
                return self.format_axiom_lets(decl.expr)
            if name_width and self.axiom_aligns(decl):
                prec = PRECEDENCE["="]
                lhs = self.expr(decl.expr.data["left"], prec).ljust(name_width)
                return f"axiom {lhs} = {self.expr(decl.expr.data['right'], prec + 1)};"
            return f"axiom {self.expr(decl.expr)};"
        if isinstance(decl, LetDecl):
            return f"let {decl.name.ljust(name_width)} = {self.expr(decl.expr)};"
        raise TypeError(f"unsupported declaration: {decl!r}")

    def format_axiom_lets(self, expr: Node) -> str:
        # A let chain at the axiom spine breaks after each `in`, with the
        # bindings and final body aligned under the first one.
        lines: list[str] = []
        current: Any = expr
        while isinstance(current, Node) and current.kind in ("let", "let_tuple"):
            lines.append(f"{self.let_binding(current)} in")
            current = current.data["body"]
        lines.append(f"{self.expr(current)};")
        indent = " " * len("axiom ")
        return "axiom " + f"\n{indent}".join(lines)

    # Type grammar precedence: sum `|` (1) < arrow `→` (2, right-assoc)
    # < product `×` (3) < primary (4). Children that bind looser than the
    # surrounding context get parenthesized so formatting preserves the AST.
    TYPE_SUM, TYPE_ARROW, TYPE_PRODUCT, TYPE_PRIMARY = 1, 2, 3, 4

    def type_expr(self, value: Any, min_prec: int = 0) -> str:
        if not isinstance(value, Node):
            raise TypeError(f"unsupported type expression: {value!r}")
        data = value.data
        if value.kind == "type_name":
            return data["name"]
        if value.kind == "type_builtin":
            return self.sym(data["name"])
        if value.kind == "type_unit":
            return "()"
        if value.kind == "type_sequence":
            return f"Seq[{self.type_expr(data['item'])}]"
        if value.kind == "type_function":
            left = self.type_expr(data["left"], self.TYPE_PRODUCT)
            right = self.type_expr(data["right"], self.TYPE_ARROW)
            return self.wrap(f"{left} {self.sym('→')} {right}", self.TYPE_ARROW, min_prec)
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
        if value.kind == "number":
            return data["value"]
        if value.kind == "string":
            return repr(data["value"])
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
        raise TypeError(f"unsupported expression: {value!r}")

    def let_binding(self, value: Node) -> str:
        if value.kind == "let_tuple":
            pattern = "(" + ", ".join(value.data["binders"]) + ")"
        else:
            pattern = value.data["name"]
        return f"let {pattern} = {self.expr(value.data['value'])}"


def format_spec(module: Module, *, ascii: bool = False, valign: bool = True) -> str:
    return Formatter(ascii=ascii, valign=valign).format_module(module)
