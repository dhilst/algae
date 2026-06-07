"""Lexer and parser for equational .alg specifications."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .ast import AxiomDecl, LemmaDecl, LetDecl, Module, OpDecl, SortDecl, VarDecl, node


@dataclass(frozen=True, slots=True)
class Token:
    kind: str
    value: str
    text: str
    line: int
    column: int


class ParseFailure(Exception):
    def __init__(self, token: Token, expected: str) -> None:
        self.token = token
        self.expected = expected
        super().__init__(expected)


KEYWORDS = {
    "sort", "op", "var", "axiom", "true", "false", "if", "then", "else", "let", "in",
    "lemma", "proof", "qed", "by",
}

WORD_SYMBOLS = {
    "product": "×",
    "arrow": "→",
    "Nat": "ℕ",
    "Int": "ℤ",
    "Real": "ℝ",
    "Bool": "𝔹",
    "not": "¬",
    "and": "∧",
    "or": "∨",
    "implies": "⟹",
    "iff": "⟺",
    "neq": "≠",
    "leq": "≤",
    "geq": "≥",
    "truth": "⊤",
    "falsehood": "⊥",
}

ASCII_SYMBOLS = {
    "->": "→",
    "-/->": "⇸",
    "==>": "⟹",
    "<==>": "⟺",
    "!=": "≠",
    "<=": "≤",
    ">=": "≥",
    "&&": "∧",
    "||": "∨",
    "/\\": "∧",
    "\\/": "∨",
    "++": "++",
    "..": "..",
    "|>": "▷",
}

UNICODE_SYMBOLS = set(WORD_SYMBOLS.values()) | {"▷", "⇸"}
ASCII_SYMBOLS_BY_LENGTH = sorted(ASCII_SYMBOLS.items(), key=lambda item: len(item[0]), reverse=True)
SINGLE_SYMBOLS = set("{}[](),;:=.+-*/<>|'")
COMPARISONS = {"=", "≠", "<", "≤", ">", "≥"}
TYPE_BUILTINS = {"ℕ", "ℤ", "ℝ", "𝔹"}
PRECEDENCE = {
    "⟺": 1,
    "⟹": 2,
    "∨": 3,
    "∧": 4,
    **{op: 5 for op in COMPARISONS},
    "▷": 6,  # pipe-last application sugar: x ▷ f(a) reads as f(a, x)
    **{op: 7 for op in ("+", "-", "++")},
    **{op: 8 for op in ("*", "/", "×")},
    ".": 9,  # pipe-first application sugar: x.f(a) reads as f(x, a)
}


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
            raw = text[start:index]
            emit("COMMENT", raw[1:].strip(), raw, line, column)
            advance(raw)
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
        tokens = lex(text)
        self.comments = [token for token in tokens if token.kind == "COMMENT"]
        self.tokens = tuple(token for token in tokens if token.kind != "COMMENT")
        self.pos = 0
        self.comment_pos = 0

    @property
    def current(self) -> Token:
        return self.tokens[self.pos]

    def advance(self) -> None:
        self.pos += 1

    def fail(self, expected: str) -> None:
        raise ParseFailure(self.current, expected)

    def consume(self, value: str, expected: str | None = None) -> Token:
        token = self.current
        if token.value == value:
            self.advance()
            return token
        self.fail(expected or value)

    def consume_keyword(self, value: str) -> Token:
        token = self.current
        if token.kind == "KEYWORD" and token.value == value:
            self.advance()
            return token
        self.fail(value)

    def consume_ident(self, expected: str = "identifier") -> str:
        token = self.current
        if token.kind == "IDENT" and token.value != "_":
            self.advance()
            return token.value
        self.fail(expected)

    def consume_binder(self) -> str:
        # `_` is a fresh anonymous variable, valid only here.
        token = self.current
        if token.kind == "IDENT":
            self.advance()
            return token.value
        self.fail("binder")

    def match(self, value: str) -> bool:
        if self.current.value == value:
            self.advance()
            return True
        return False

    def take_comments_before(self, line: int) -> list[str]:
        taken: list[str] = []
        while self.comment_pos < len(self.comments) and self.comments[self.comment_pos].line < line:
            taken.append(self.comments[self.comment_pos].value)
            self.comment_pos += 1
        return taken

    def parse(self) -> Module:
        try:
            declarations: list[Any] = []
            while self.current.kind != "EOF":
                leading = self.take_comments_before(self.current.line)
                start_line = self.current.line
                decl = self.parse_decl()
                decl.line = start_line
                end_line = self.tokens[self.pos - 1].line
                # Comments inside a multi-line declaration are hoisted above it.
                leading.extend(self.take_comments_before(end_line))
                decl.leading_comments = leading
                if self.comment_pos < len(self.comments) and self.comments[self.comment_pos].line == end_line:
                    decl.trailing_comment = self.comments[self.comment_pos].value
                    self.comment_pos += 1
                declarations.append(decl)
            trailing = [token.value for token in self.comments[self.comment_pos :]]
            return Module(declarations, trailing_comments=trailing)
        except ParseFailure as exc:
            token = exc.token
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
        if token.value == "lemma":
            return self.parse_lemma()
        if token.value == "let":
            return self.parse_let()
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
        partial = False
        if self.match("⇸"):
            partial = True
        elif not self.match("→"):
            domain = self.parse_type_product_items()
            if self.current.value == "|":
                # A top-level `|` folds the domain into a single sum-typed
                # argument: `A × B | C → D` takes one argument of type (A×B)|C,
                # grouping as in codomains. Branches are products; an arrow in
                # a branch needs parens so the signature arrow stays visible.
                head = domain[0] if len(domain) == 1 else node("type_product", items=domain)
                items = [head]
                while self.match("|"):
                    items.append(self.parse_type_product())
                domain = [node("type_sum", items=items)]
            if self.match("⇸"):
                partial = True
            else:
                self.consume("→", "-> or -/->")
        codomain = self.parse_type_expr()
        self.consume(";")
        return OpDecl(name, domain, codomain, partial)

    def parse_var(self) -> VarDecl:
        # `var e, f : Elem;` declares every name at the same sort.
        self.consume_keyword("var")
        names = [self.consume_ident("variable name")]
        while self.match(","):
            names.append(self.consume_ident("variable name"))
        self.consume(":")
        sort = self.parse_type_expr()
        self.consume(";")
        return VarDecl(names, sort)

    def parse_axiom(self) -> AxiomDecl:
        self.consume_keyword("axiom")
        name = self.parse_rule_name("axiom name")
        expr = self.parse_expr()
        self.consume(";")
        return AxiomDecl(expr, name)

    def parse_rule_name(self, expected: str) -> str:
        # Axiom, lemma, and `by` rule names are identifiers with trailing
        # primes allowed, e.g. assoc'.
        name = self.consume_ident(expected)
        while self.match("'"):
            name += "'"
        return name

    def parse_lemma(self) -> LemmaDecl:
        # `lemma name expr;` optionally followed by a proof block. Both are
        # parsed and stored only; nothing is checked or proved yet.
        self.consume_keyword("lemma")
        name = self.parse_rule_name("lemma name")
        expr = self.parse_expr()
        self.consume(";")
        proof = None
        if self.current.kind == "KEYWORD" and self.current.value == "proof":
            proof = self.parse_proof()
        return LemmaDecl(expr, name, proof)

    def parse_proof(self) -> Any:
        # proof_step ::= expr ';' | '=' expr 'by' rule_name ';'
        self.consume_keyword("proof")
        steps: list[Any] = []
        while not (self.current.kind == "KEYWORD" and self.current.value == "qed"):
            if self.match("="):
                expr = self.parse_expr()
                self.consume_keyword("by")
                rule = self.parse_rule_name("rule name")
                self.consume(";")
                steps.append(node("proof_rewrite", expr=expr, rule=rule))
            else:
                expr = self.parse_expr()
                self.consume(";")
                steps.append(node("proof_start", expr=expr))
        self.consume_keyword("qed")
        self.consume(";")
        return node("proof", steps=steps)

    def parse_let(self) -> LetDecl:
        self.consume_keyword("let")
        name = self.consume_ident("let name")
        self.consume("=")
        expr = self.parse_expr()
        self.consume(";")
        return LetDecl(name, expr)

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
            return node("type_function", left=left, right=right, partial=False)
        if self.match("⇸"):
            right = self.parse_type_arrow()
            return node("type_function", left=left, right=right, partial=True)
        return left

    def parse_type_product(self) -> Any:
        parts = self.parse_type_product_items()
        if len(parts) == 1:
            return parts[0]
        return node("type_product", items=parts)

    def parse_type_product_items(self) -> list[Any]:
        # `*` is the symbolic ASCII alias for `×` in type position; the lexer
        # cannot canonicalize it because `*` is also multiplication.
        parts = [self.parse_type_primary()]
        while self.match("×") or self.match("*"):
            parts.append(self.parse_type_primary())
        return parts

    def parse_type_primary(self) -> Any:
        token = self.current
        if token.kind == "IDENT":
            name = self.consume_ident()
            if name == "Seq" and self.match("["):
                item = self.parse_type_expr()
                self.consume("]")
                return node("type_sequence", item=item)
            return node("type_name", name=name)
        if token.value in TYPE_BUILTINS:
            self.advance()
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
            prec = PRECEDENCE.get(op, -1)
            if prec < min_prec:
                break
            self.advance()
            right_min = prec if op in {"⟹", "⟺"} else prec + 1
            right = self.parse_binary(right_min)
            left = node("binary", op=op, left=left, right=right)
        return left

    def parse_prefix(self) -> Any:
        token = self.current
        if token.value in {"¬", "-"}:
            self.advance()
            return node("unary", op=token.value, value=self.parse_binary(9))
        if token.kind == "KEYWORD" and token.value == "if":
            self.advance()
            condition = self.parse_expr()
            self.consume_keyword("then")
            then_expr = self.parse_expr()
            self.consume_keyword("else")
            else_expr = self.parse_expr()
            return node("if", condition=condition, then=then_expr, otherwise=else_expr)
        if token.kind == "KEYWORD" and token.value == "let":
            self.advance()
            if self.current.value == "(":
                self.consume("(")
                binders = [self.consume_binder()]
                self.consume(",", "',' (a destructuring pattern needs at least two binders)")
                binders.append(self.consume_binder())
                while self.match(","):
                    binders.append(self.consume_binder())
                self.consume(")")
                self.consume("=")
                value = self.parse_expr()
                self.consume_keyword("in")
                body = self.parse_expr()
                return node("let_tuple", binders=binders, value=value, body=body)
            name = self.consume_ident("let variable")
            self.consume("=")
            value = self.parse_expr()
            self.consume_keyword("in")
            body = self.parse_expr()
            return node("let", name=name, value=value, body=body)
        return self.parse_postfix(self.parse_atom())

    def parse_atom(self) -> Any:
        token = self.current
        if token.kind == "IDENT":
            if token.value == "_":
                self.fail("expression ('_' is only valid in destructuring patterns)")
            self.advance()
            return node("identifier", name=token.value)
        if token.kind == "NUMBER":
            self.advance()
            return node("number", value=token.value)
        if token.kind == "STRING":
            self.advance()
            return node("string", value=token.value)
        if token.kind == "KEYWORD" and token.value in {"true", "false"}:
            self.advance()
            return node("bool", value=(token.value == "true"))
        if token.value in {"⊤", "⊥"}:
            self.advance()
            return node("bool_symbol", value=token.value)
        if token.value in TYPE_BUILTINS:
            self.advance()
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


def parse_text(text: str) -> Module:
    return AlgParser(text).parse()


def parse_file(path: str | Path) -> Module:
    return parse_text(Path(path).read_text(encoding="utf-8"))
