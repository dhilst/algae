"""Lexer and parser for equational .alg specifications."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .ast import AxiomDecl, Module, OpDecl, SortDecl, VarDecl, node
from .combinators import ParseFailure, State, Token, token_kind, token_value


KEYWORDS = {
    "sort",
    "op",
    "var",
    "axiom",
    "true",
    "false",
    "if",
    "then",
    "else",
    "spec",
    "state",
    "init",
    "inv",
    "pre",
    "post",
    "ret",
    "prop",
    "fn",
    "import",
    "extends",
    "type",
}

WORD_SYMBOLS = {
    "in": "∈",
    "notin": "∉",
    "subseteq": "⊆",
    "subset": "⊂",
    "superseteq": "⊇",
    "superset": "⊃",
    "union": "∪",
    "intersect": "∩",
    "setminus": "\\",
    "product": "×",
    "arrow": "→",
    "mapsto": "↦",
    "emptyset": "∅",
    "nat": "ℕ",
    "int": "ℤ",
    "real": "ℝ",
    "bool": "𝔹",
    "forall": "∀",
    "exists": "∃",
    "not": "¬",
    "and": "∧",
    "or": "∨",
    "implies": "⟹",
    "iff": "⟺",
    "powerset": "℘",
    "dot": "·",
    "neq": "≠",
    "leq": "≤",
    "geq": "≥",
    "truth": "⊤",
    "falsehood": "⊥",
    "override": "⊕",
    "bigunion": "⋃",
}

ASCII_SYMBOLS = {
    "->": "→",
    "|->": "↦",
    "==>": "⟹",
    "<==>": "⟺",
    "!=": "≠",
    "<=": "≤",
    ">=": "≥",
    "&&": "∧",
    "||": "∨",
    "++": "++",
    "..": "..",
}

UNICODE_SYMBOLS = set(WORD_SYMBOLS.values())
ASCII_SYMBOLS_BY_LENGTH = sorted(ASCII_SYMBOLS.items(), key=lambda item: len(item[0]), reverse=True)
SINGLE_SYMBOLS = set("{}[](),;:=.+-*/<>|\\'")
COMPARISONS = {"=", "≠", "<", "≤", ">", "≥", "∈", "∉", "⊆", "⊂", "⊇", "⊃"}
TYPE_BUILTINS = {"ℕ", "ℤ", "ℝ", "𝔹"}
OLD_KEYWORDS = {"spec", "state", "init", "inv", "pre", "post", "ret", "prop", "fn", "import", "extends", "type"}


@dataclass(slots=True)
class ParseError(Exception):
    line: int
    column: int
    expected: str
    found: str

    def __str__(self) -> str:
        return f"error at {self.line}, Expected {self.expected} found {self.found}"


def lex(text: str) -> tuple[Token, ...]:
    tokens: list[Token] = []
    index = 0
    line = 1
    column = 1

    def emit(kind: str, value: str, raw: str, start_line: int, start_col: int) -> None:
        tokens.append(Token(kind, value, raw, start_line, start_col))

    def advance(raw: str) -> None:
        nonlocal line, column
        for char in raw:
            if char == "\n":
                line += 1
                column = 1
            else:
                column += 1

    while index < len(text):
        char = text[index]
        if char.isspace():
            advance(char)
            index += 1
            continue
        if char == "#":
            start = index
            while index < len(text) and text[index] != "\n":
                index += 1
            advance(text[start:index])
            continue
        start_line, start_col = line, column
        matched = None
        for raw, canonical in ASCII_SYMBOLS_BY_LENGTH:
            if text.startswith(raw, index):
                matched = (raw, canonical)
                break
        if matched:
            raw, canonical = matched
            emit("SYMBOL", canonical, raw, start_line, start_col)
            advance(raw)
            index += len(raw)
            continue
        if char in UNICODE_SYMBOLS:
            emit("SYMBOL", char, char, start_line, start_col)
            advance(char)
            index += 1
            continue
        if char in SINGLE_SYMBOLS:
            emit("SYMBOL", char, char, start_line, start_col)
            advance(char)
            index += 1
            continue
        if char.isdigit():
            start = index
            while index < len(text) and text[index].isdigit():
                index += 1
            raw = text[start:index]
            emit("NUMBER", raw, raw, start_line, start_col)
            advance(raw)
            continue
        if char == '"':
            start = index
            index += 1
            escaped = False
            while index < len(text):
                current = text[index]
                index += 1
                if escaped:
                    escaped = False
                elif current == "\\":
                    escaped = True
                elif current == '"':
                    break
            else:
                raw = text[start:index]
                emit("ERROR", raw, raw, start_line, start_col)
                advance(raw)
                continue
            raw = text[start:index]
            emit("STRING", raw[1:-1], raw, start_line, start_col)
            advance(raw)
            continue
        if char.isalpha() or char == "_":
            start = index
            while index < len(text) and (text[index].isalnum() or text[index] == "_"):
                index += 1
            raw = text[start:index]
            if raw in WORD_SYMBOLS:
                emit("SYMBOL", WORD_SYMBOLS[raw], raw, start_line, start_col)
            elif raw in KEYWORDS:
                emit("KEYWORD", raw, raw, start_line, start_col)
            else:
                emit("IDENT", raw, raw, start_line, start_col)
            advance(raw)
            continue
        emit("ERROR", char, char, start_line, start_col)
        advance(char)
        index += 1

    tokens.append(Token("EOF", "EOF", "EOF", line, column))
    return tuple(tokens)


class AlgParser:
    def __init__(self, text: str) -> None:
        self.state = State(lex(text))
        self.ident_parser = token_kind("IDENT", "identifier")
        self.eof_parser = token_kind("EOF", "end of file")

    @property
    def current(self) -> Token:
        return self.state.current

    def fail(self, expected: str) -> None:
        raise ParseFailure(self.state, expected)

    def consume(self, value: str, expected: str | None = None) -> Token:
        token, self.state = token_value(value, expected or value)(self.state)
        return token

    def consume_keyword(self, value: str) -> Token:
        token = self.current
        if token.kind == "KEYWORD" and token.value == value:
            self.state = self.state.advance()
            return token
        self.fail(value)

    def consume_ident(self, expected: str = "identifier") -> str:
        token, self.state = self.ident_parser.label(expected)(self.state)
        return token.value

    def match(self, value: str) -> bool:
        if self.current.value == value:
            self.state = self.state.advance()
            return True
        return False

    def parse(self) -> Module:
        try:
            declarations: list[Any] = []
            while self.current.kind != "EOF":
                declarations.append(self.parse_decl())
            self.eof_parser(self.state)
            return Module(declarations)
        except ParseFailure as exc:
            token = exc.state.current
            found = "end of file" if token.kind == "EOF" else token.text
            raise ParseError(token.line, token.column, exc.expected, found) from exc

    def parse_decl(self) -> Any:
        token = self.current
        if token.kind != "KEYWORD":
            self.fail("declaration")
        if token.value == "sort":
            return self.parse_sort()
        if token.value == "op":
            return self.parse_op()
        if token.value == "var":
            return self.parse_var()
        if token.value == "axiom":
            return self.parse_axiom()
        if token.value in OLD_KEYWORDS:
            self.fail("sort, op, var, or axiom")
        self.fail("declaration")

    def parse_sort(self) -> SortDecl:
        self.consume_keyword("sort")
        names = [self.consume_ident("sort name")]
        while self.match(","):
            names.append(self.consume_ident("sort name"))
        values = None
        if self.match("="):
            if len(names) != 1:
                self.fail("single sort before enum definition")
            self.consume("{")
            values = []
            if not self.match("}"):
                while True:
                    values.append(self.consume_ident("enum value"))
                    if self.match("}"):
                        break
                    self.consume(",")
        self.consume(";")
        return SortDecl(names, values)

    def parse_op(self) -> OpDecl:
        self.consume_keyword("op")
        name = self.consume_ident("operation name")
        self.consume(":")
        domain: list[Any] = []
        if not self.match("→"):
            domain = self.parse_type_product_items()
            self.consume("→", "->")
        codomain = self.parse_type_expr()
        self.consume(";")
        return OpDecl(name, domain, codomain)

    def parse_var(self) -> VarDecl:
        self.consume_keyword("var")
        name = self.consume_ident("variable name")
        self.consume(":")
        sort = self.parse_type_expr()
        self.consume(";")
        return VarDecl(name, sort)

    def parse_axiom(self) -> AxiomDecl:
        self.consume_keyword("axiom")
        expr = self.parse_expr()
        self.consume(";")
        return AxiomDecl(expr)

    def parse_type_expr(self) -> Any:
        return self.parse_type_sum()

    def parse_type_sum(self) -> Any:
        items = [self.parse_type_arrow()]
        while self.match("|"):
            items.append(self.parse_type_arrow())
        if len(items) == 1:
            return items[0]
        return node("type_sum", items=items)

    def parse_type_arrow(self) -> Any:
        left = self.parse_type_product()
        if self.match("→"):
            right = self.parse_type_arrow()
            return node("type_function", left=left, right=right)
        return left

    def parse_type_product(self) -> Any:
        parts = self.parse_type_product_items()
        if len(parts) == 1:
            return parts[0]
        return node("type_product", items=parts)

    def parse_type_product_items(self) -> list[Any]:
        parts = [self.parse_type_primary()]
        while self.match("×"):
            parts.append(self.parse_type_primary())
        return parts

    def parse_type_primary(self) -> Any:
        token = self.current
        if self.match("℘"):
            self.consume("(")
            inner = self.parse_type_expr()
            self.consume(")")
            return node("type_powerset", item=inner)
        if token.kind == "IDENT":
            name = self.consume_ident()
            if name == "Seq" and self.match("["):
                item = self.parse_type_expr()
                self.consume("]")
                return node("type_sequence", item=item)
            return node("type_name", name=name)
        if token.value in TYPE_BUILTINS:
            self.state = self.state.advance()
            return node("type_builtin", name=token.value)
        if self.match("("):
            if self.match(")"):
                return node("type_unit")
            inner = self.parse_type_expr()
            self.consume(")")
            return inner
        self.fail("type expression")

    def parse_expr(self) -> Any:
        return self.parse_binary(0)

    def parse_binary(self, min_prec: int) -> Any:
        left = self.parse_prefix()
        while True:
            token = self.current
            op = token.value
            prec = self.precedence(op)
            if prec < min_prec:
                break
            self.state = self.state.advance()
            right_min = prec if op in {"⟹", "⟺"} else prec + 1
            right = self.parse_binary(right_min)
            left = node("binary", op=op, left=left, right=right)
        return left

    def precedence(self, op: str) -> int:
        if op == "⟺":
            return 1
        if op == "⟹":
            return 2
        if op == "∨":
            return 3
        if op == "∧":
            return 4
        if op in COMPARISONS:
            return 5
        if op in {"∪", "∩", "\\", "⊕"}:
            return 6
        if op in {"+", "-", "++"}:
            return 7
        if op in {"*", "/", "×"}:
            return 8
        return -1

    def parse_prefix(self) -> Any:
        token = self.current
        if token.value in {"¬", "-", "⋃"}:
            self.state = self.state.advance()
            return node("unary", op=token.value, value=self.parse_binary(9))
        if token.value in {"∀", "∃"}:
            self.state = self.state.advance()
            var = self.consume_ident("quantifier variable")
            self.consume("∈")
            source = self.parse_expr()
            if not (self.match("·") or self.match(":")):
                self.fail("quantifier separator")
            body = self.parse_expr()
            return node("quantifier", op=token.value, var=var, source=source, body=body)
        if token.kind == "KEYWORD" and token.value == "if":
            self.state = self.state.advance()
            condition = self.parse_expr()
            self.consume_keyword("then")
            then_expr = self.parse_expr()
            self.consume_keyword("else")
            else_expr = self.parse_expr()
            return node("if", condition=condition, then=then_expr, otherwise=else_expr)
        return self.parse_postfix(self.parse_atom())

    def parse_atom(self) -> Any:
        token = self.current
        if token.kind == "IDENT":
            self.state = self.state.advance()
            return node("identifier", name=token.value)
        if token.kind == "NUMBER":
            self.state = self.state.advance()
            return node("number", value=token.value)
        if token.kind == "STRING":
            self.state = self.state.advance()
            return node("string", value=token.value)
        if token.kind == "KEYWORD" and token.value in {"true", "false"}:
            self.state = self.state.advance()
            return node("bool", value=(token.value == "true"))
        if token.value in {"⊤", "⊥"}:
            self.state = self.state.advance()
            return node("bool_symbol", value=token.value)
        if token.value == "∅":
            self.state = self.state.advance()
            return node("empty")
        if token.value in TYPE_BUILTINS:
            self.state = self.state.advance()
            return node("builtin_set", name=token.value)
        if self.match("("):
            if self.match(")"):
                return node("unit")
            first = self.parse_expr()
            if self.match(","):
                items = [first]
                while True:
                    items.append(self.parse_expr())
                    if self.match(")"):
                        return node("tuple", items=items)
                    self.consume(",")
            self.consume(")")
            return first
        if self.match("{"):
            return self.parse_braced_expr()
        self.fail("expression")

    def parse_postfix(self, expr: Any) -> Any:
        while True:
            if self.match("("):
                args = self.parse_expr_list(")")
                expr = node("call", function=expr, args=args)
            elif self.match("'"):
                expr = node("prime", value=expr)
            else:
                return expr

    def parse_expr_list(self, closer: str) -> list[Any]:
        items: list[Any] = []
        if self.match(closer):
            return items
        while True:
            items.append(self.parse_expr())
            if self.match(closer):
                return items
            self.consume(",")

    def parse_braced_expr(self) -> Any:
        if self.match("}"):
            return node("set", items=[])
        first = self.parse_expr()
        if self.match("↦"):
            value = self.parse_expr()
            self.consume("}")
            return node("mapping", key=first, value=value)
        items = [first]
        while self.match(","):
            items.append(self.parse_expr())
        self.consume("}")
        return node("set", items=items)


def parse_text(text: str) -> Module:
    return AlgParser(text).parse()


def parse_file(path: str | Path) -> Module:
    return parse_text(Path(path).read_text(encoding="utf-8"))
