"""Formatter for equational .alg ASTs."""

from __future__ import annotations

from typing import Any

from .ast import AxiomDecl, LetDecl, Module, Node, OpDecl, SortDecl, VarDecl
from .parser import WORD_SYMBOLS

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
                    len(self.expr(decl.expr.data["left"]))
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
            domain = f" {self.sym('×')} ".join(self.type_expr(item) for item in decl.domain)
            if domain:
                return f"op {name} : {domain} {self.sym('→')} {self.type_expr(decl.codomain)};"
            return f"op {name} : {self.sym('→')} {self.type_expr(decl.codomain)};"
        if isinstance(decl, VarDecl):
            return f"var {decl.name.ljust(name_width)} : {self.type_expr(decl.sort)};"
        if isinstance(decl, AxiomDecl):
            if isinstance(decl.expr, Node) and decl.expr.kind in ("let", "let_tuple"):
                return self.format_axiom_lets(decl.expr)
            if name_width and self.axiom_aligns(decl):
                lhs = self.expr(decl.expr.data["left"]).ljust(name_width)
                return f"axiom {lhs} = {self.expr(decl.expr.data['right'])};"
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

    def type_expr(self, value: Any) -> str:
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
            return f"{self.type_expr(data['left'])} {self.sym('→')} {self.type_expr(data['right'])}"
        if value.kind == "type_product":
            return f" {self.sym('×')} ".join(self.type_expr(item) for item in data["items"])
        if value.kind == "type_sum":
            return " | ".join(self.type_expr(item) for item in data["items"])
        raise TypeError(f"unsupported type expression: {value!r}")

    def expr(self, value: Any) -> str:
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
            return self.expr(data["value"]) + "'"
        if value.kind == "call":
            return self.expr(data["function"]) + "(" + ", ".join(self.expr(arg) for arg in data["args"]) + ")"
        if value.kind == "unary":
            return self.sym(data["op"]) + " " + self.expr(data["value"])
        if value.kind == "binary":
            if data["op"] == ".":
                return f"{self.expr(data['left'])}.{self.expr(data['right'])}"
            return f"{self.expr(data['left'])} {self.sym(data['op'])} {self.expr(data['right'])}"
        if value.kind == "if":
            return (
                f"if {self.expr(data['condition'])} then {self.expr(data['then'])} "
                f"else {self.expr(data['otherwise'])}"
            )
        if value.kind in ("let", "let_tuple"):
            return f"{self.let_binding(value)} in {self.expr(data['body'])}"
        raise TypeError(f"unsupported expression: {value!r}")

    def let_binding(self, value: Node) -> str:
        if value.kind == "let_tuple":
            pattern = "(" + ", ".join(value.data["binders"]) + ")"
        else:
            pattern = value.data["name"]
        return f"let {pattern} = {self.expr(value.data['value'])}"


def format_spec(module: Module, *, ascii: bool = False, valign: bool = True) -> str:
    return Formatter(ascii=ascii, valign=valign).format_module(module)
