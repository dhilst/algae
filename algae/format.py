"""Formatter for equational .alg ASTs."""

from __future__ import annotations

from typing import Any

from .ast import AxiomDecl, Module, Node, OpDecl, SortDecl, VarDecl
from .parser import WORD_SYMBOLS

ASCII = {symbol: word for word, symbol in WORD_SYMBOLS.items()}


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
            lines.append(self.format_decl(decl))
            previous_group = group
        return "\n".join(lines) + ("\n" if lines else "")

    def format_decl(self, decl: Any) -> str:
        if isinstance(decl, SortDecl):
            if decl.values is not None:
                return f"sort {decl.names[0]} = " + "{" + ", ".join(decl.values) + "};"
            return f"sort {', '.join(decl.names)};"
        if isinstance(decl, OpDecl):
            domain = f" {self.sym('×')} ".join(self.type_expr(item) for item in decl.domain)
            if domain:
                return f"op {decl.name} : {domain} {self.sym('→')} {self.type_expr(decl.codomain)};"
            return f"op {decl.name} : {self.sym('→')} {self.type_expr(decl.codomain)};"
        if isinstance(decl, VarDecl):
            return f"var {decl.name} : {self.type_expr(decl.sort)};"
        if isinstance(decl, AxiomDecl):
            return f"axiom {self.expr(decl.expr)};"
        raise TypeError(f"unsupported declaration: {decl!r}")

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
        if value.kind == "type_powerset":
            return f"{self.sym('℘')}({self.type_expr(data['item'])})"
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
        if value.kind == "empty":
            return self.sym("∅")
        if value.kind == "builtin_set":
            return self.sym(data["name"])
        if value.kind == "unit":
            return "()"
        if value.kind == "tuple":
            return "(" + ", ".join(self.expr(item) for item in data["items"]) + ")"
        if value.kind == "set":
            return "{" + ", ".join(self.expr(item) for item in data["items"]) + "}"
        if value.kind == "mapping":
            return "{" + f"{self.expr(data['key'])} {self.sym('↦')} {self.expr(data['value'])}" + "}"
        if value.kind == "prime":
            return self.expr(data["value"]) + "'"
        if value.kind == "call":
            return self.expr(data["function"]) + "(" + ", ".join(self.expr(arg) for arg in data["args"]) + ")"
        if value.kind == "unary":
            return self.sym(data["op"]) + " " + self.expr(data["value"])
        if value.kind == "binary":
            return f"{self.expr(data['left'])} {self.sym(data['op'])} {self.expr(data['right'])}"
        if value.kind == "quantifier":
            return (
                f"{self.sym(data['op'])} {data['var']} {self.sym('∈')} "
                f"{self.expr(data['source'])} {self.sym('·')} {self.expr(data['body'])}"
            )
        if value.kind == "if":
            return (
                f"if {self.expr(data['condition'])} then {self.expr(data['then'])} "
                f"else {self.expr(data['otherwise'])}"
            )
        if value.kind == "let":
            return f"let {data['name']} = {self.expr(data['value'])} in {self.expr(data['body'])}"
        raise TypeError(f"unsupported expression: {value!r}")


def format_spec(module: Module, *, ascii: bool = False) -> str:
    return Formatter(ascii=ascii).format_module(module)
