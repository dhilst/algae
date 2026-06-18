"""Lexer and parser for equational .alg specifications."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .ast import (
    AliasDecl,
    EqDecl,
    IncludeDecl,
    LemmaDecl,
    LetDecl,
    Module,
    OpDecl,
    OpenDecl,
    ParamDecl,
    PropDecl,
    RuleDecl,
    SortDecl,
    node,
)


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
    "sort", "param", "op", "eq", "prop", "true", "false", "if", "then", "else",
    "let", "in", "lemma", "proof", "qed", "wip", "by", "rule", "apply",
    "case", "end", "st", "goal", "rewrite", "therefore", "done",
    "include", "open", "with", "alias", "props",
}

WORD_SYMBOLS = {
    "product": "×",
    "arrow": "→",
    "Bool": "𝔹",
    "Prop": "Prop",  # built-in proposition type; no separate Unicode glyph
    "Sort": "Sort",  # the kind of sorts; no separate Unicode glyph
    "not": "¬",
    "and": "∧",
    "or": "∨",
    "implies": "⟹",
    "iff": "⟺",
    "neq": "≠",
    "truth": "⊤",
    "falsehood": "⊥",
    "fun": "λ",
    "forall": "∀",
    "exists": "∃",
}

ASCII_SYMBOLS = {
    "->": "→",
    "-/->": "⇸",
    "==>": "⟹",
    "<==>": "⟺",
    "!=": "≠",
    "&&": "∧",
    "||": "∨",
    "/\\": "∧",
    "\\/": "∨",
    "++": "++",
    "|>": "▷",
    "|-": "⊢",
    ":=": ":=",
    "=>": "=>",
    "::": "::",
}

UNICODE_SYMBOLS = set(WORD_SYMBOLS.values()) | {"▷", "⇸", "⊢"}
ASCII_SYMBOLS_BY_LENGTH = sorted(ASCII_SYMBOLS.items(), key=lambda item: len(item[0]), reverse=True)
SINGLE_SYMBOLS = set("{}[](),;:=.+-*/<>|'")
COMPARISONS = {"=", "≠"}
TYPE_BUILTINS = {"𝔹", "Prop", "Sort"}
PRECEDENCE = {
    "⟺": 1,
    "⟹": 2,
    "∨": 3,
    "∧": 4,
    **{op: 5 for op in COMPARISONS},
    "▷": 6,  # pipe-last application sugar: x ▷ f(a) reads as f(a, x)
    "++": 7,  # sequence concatenation
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
        if char == "─":
            # A run of box-drawing dashes is the rule bar separating premises
            # from the conclusion. Any length collapses to one RULE_BAR token.
            start = index
            while index < len(text) and text[index] == "─":
                index += 1
            raw = text[start:index]
            emit("RULE_BAR", "─", raw, line, column)
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
        if token.value == "param":
            return self.parse_param()
        if token.value == "op":
            return self.parse_op()
        if token.value == "eq":
            return self.parse_eq()
        if token.value == "prop":
            return self.parse_prop_decl()
        if token.value == "lemma":
            return self.parse_lemma()
        if token.value == "rule":
            return self.parse_rule()
        if token.value == "include":
            return self.parse_include()
        if token.value == "open":
            return self.parse_open()
        if token.value == "alias":
            return self.parse_alias()
        if token.value == "let":
            return self.parse_let()
        self.fail("declaration")

    def parse_module_path(self) -> list[str]:
        # foo::bar::baz  →  ["foo", "bar", "baz"]
        parts = [self.consume_ident("module name")]
        while self.match("::"):
            parts.append(self.consume_ident("module name"))
        return parts

    def parse_include(self) -> IncludeDecl:
        # include foo::bar with (T := type, op := name) props case prop ... qed; qed;
        self.consume_keyword("include")
        path = self.parse_module_path()
        bindings: list[Any] = []
        if self.current.kind == "KEYWORD" and self.current.value == "with":
            self.advance()
            self.consume("(")
            while True:
                name = self.consume_ident("parameter name")
                self.consume(":=")
                bindings.append((name, self.parse_type_expr()))
                if self.match(")"):
                    break
                self.consume(",")
        # Obligation discharge: `props <case ...>* <qed|wip>`. The block is a
        # subproof, so it ends with a terminator (wip when any case is wip).
        obligations: list[Any] = []
        terminator: str | None = None
        if self.current.kind == "KEYWORD" and self.current.value == "props":
            self.advance()
            while self.current.kind == "KEYWORD" and self.current.value == "case":
                obligations.append(self.parse_case())
            terminator = self.consume_terminator()
        self.consume(";")
        return IncludeDecl(path, bindings, obligations, terminator)

    def parse_open(self) -> OpenDecl:
        # open foo::bar (nil, cons);   (the name list is required)
        self.consume_keyword("open")
        path = self.parse_module_path()
        self.consume("(")
        names = [self.consume_ident("imported name")]
        while self.match(","):
            names.append(self.consume_ident("imported name"))
        self.consume(")")
        self.consume(";")
        return OpenDecl(path, names)

    def parse_alias(self) -> AliasDecl:
        # alias bar = foo::bar;
        self.consume_keyword("alias")
        name = self.consume_ident("alias name")
        self.consume("=")
        path = self.parse_module_path()
        self.consume(";")
        return AliasDecl(name, path)

    def parse_sort(self) -> SortDecl:
        # sort Name : Kind;   where Kind is Sort, Sort → Sort, …
        self.consume_keyword("sort")
        name = self.consume_ident("sort name")
        self.consume(":", ": (a sort needs a kind, e.g. `sort Nat : Sort;`)")
        kind = self.parse_type_expr()
        self.consume(";")
        return SortDecl(name, kind)

    def parse_param(self) -> ParamDecl:
        # param Name : Kind;   an abstract sort or sort constructor
        self.consume_keyword("param")
        name = self.consume_ident("parameter name")
        self.consume(":", ": (a param needs a kind, e.g. `param T : Sort;`)")
        kind = self.parse_type_expr()
        self.consume(";")
        return ParamDecl(name, kind)

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

    def parse_eq(self) -> EqDecl:
        # eq name (binders) lhs = rhs;   a trusted equation
        self.consume_keyword("eq")
        name = self.parse_rule_name("equation name")
        params, expr = self.parse_equation_decl()
        self.consume(";")
        return EqDecl(expr, name, params=params)

    def parse_prop_decl(self) -> PropDecl:
        # prop name (binders) lhs = rhs;   an instantiation obligation
        self.consume_keyword("prop")
        name = self.parse_rule_name("proposition name")
        params, expr = self.parse_equation_decl()
        self.consume(";")
        return PropDecl(expr, name, params=params)

    def parse_lemma(self) -> LemmaDecl:
        # lemma name (binders) lhs = rhs;  optionally followed by a proof block.
        # The proposition is checked; the proof is parsed and stored only.
        self.consume_keyword("lemma")
        name = self.parse_rule_name("lemma name")
        params, expr = self.parse_equation_decl()
        self.consume(";")
        proof = None
        if self.current.kind == "KEYWORD" and self.current.value == "proof":
            proof = self.parse_proof()
        return LemmaDecl(expr, name, proof, params=params)

    def parse_equation_decl(self) -> tuple[list[Any], Any]:
        # The body of an eq/prop/lemma: optional binders, then an equation.
        # Binders are schematic parameters; the body is not a sequent or forall.
        params = self.parse_binder_list() if self.looks_like_binder_list() else []
        return params, self.parse_expr()

    def parse_rule_name(self, expected: str) -> str:
        # eq, prop, lemma, rule, and theorem names are identifiers with trailing
        # primes allowed, e.g. assoc'.
        name = self.consume_ident(expected)
        while self.match("'"):
            name += "'"
        return name

    def parse_prop(self) -> Any:
        # prop ::= expr | sequent ; sequent ::= context? '⊢' expr
        # A bare expr is its own proposition; a sequent carries a context.
        if self.match("⊢"):
            return node("sequent", assumptions=[], goal=self.parse_expr())
        first = self.parse_assumption()
        if self.current.value in (",", "⊢"):
            assumptions = [first]
            while self.match(","):
                assumptions.append(self.parse_assumption())
            self.consume("⊢", "⊢ (turnstile)")
            return node("sequent", assumptions=assumptions, goal=self.parse_expr())
        if first.kind == "context_var":
            self.fail("⊢ (turnstile) after a typed context variable")
        if first.data["name"] is not None:
            self.fail("⊢ (turnstile) after a named assumption")
        return first.data["expr"]

    def parse_assumption(self) -> Any:
        # context entry ::= x ':' type | identifier ':=' expr | expr
        if self.current.kind == "IDENT":
            nxt = self.tokens[self.pos + 1].value
            if nxt == ":=":
                name = self.consume_ident("assumption name")
                self.consume(":=")
                return node("assumption", name=name, expr=self.parse_expr())
            if nxt == ":":
                name = self.consume_ident("variable name")
                self.consume(":")
                return node("context_var", name=name, type=self.parse_type_expr())
        return node("assumption", name=None, expr=self.parse_expr())

    def parse_binder_list(self) -> list[Any]:
        # ( name+ : type (',' name+ : type)* )  →  flat list of (name, type).
        # Co-typed names are space-separated (`b b' : B`); entries are
        # comma-separated. Shared by λ, quantifiers, rule and eq/prop/lemma params.
        self.consume("(")
        binders: list[Any] = []
        if self.match(")"):
            return binders
        while True:
            names = [self.parse_rule_name("binder name")]
            while self.current.kind == "IDENT":
                names.append(self.parse_rule_name("binder name"))
            self.consume(":")
            btype = self.parse_type_expr()
            binders.extend((name, btype) for name in names)
            if self.match(")"):
                break
            self.consume(",")
        return binders

    def looks_like_binder_list(self) -> bool:
        # True when the upcoming `( ... )` opens a binder list (`( IDENT+ :`),
        # not an expression that merely starts with `(` such as a tuple body.
        if self.current.value != "(":
            return False
        index = self.pos + 1
        if self.tokens[index].kind != "IDENT":
            return False
        while self.tokens[index].kind == "IDENT":
            index += 1
            if self.tokens[index].value == "'":  # primed binder name
                index += 1
        return self.tokens[index].value == ":"

    def parse_rule(self) -> RuleDecl:
        # rule name(params) (case name <sequent> end;)* ───── conclusion end;
        self.consume_keyword("rule")
        name = self.parse_rule_name("rule name")
        params = self.parse_binder_list()
        premises: list[Any] = []
        while self.current.kind == "KEYWORD" and self.current.value == "case":
            premises.append(self.parse_rule_premise())
        self.consume("─", "rule bar (─────)")
        conclusion = self.parse_prop()
        self.consume_keyword("end")
        self.consume(";")
        return RuleDecl(name, params, premises, conclusion)

    def parse_rule_premise(self) -> Any:
        # case name <sequent> end;
        self.consume_keyword("case")
        name = self.consume_ident("premise case name")
        prop = self.parse_prop()
        self.consume_keyword("end")
        self.consume(";")
        return node("rule_premise", name=name, prop=prop)

    def parse_proof(self) -> Any:
        self.consume_keyword("proof")
        steps = self.parse_proof_steps()
        terminator = self.consume_terminator()
        self.consume(";")
        return node("proof", steps=steps, terminator=terminator)

    # A subproof (proof block, case, apply block, include obligations) ends with
    # `qed` or, when it is still work in progress (uses the `wip` tactic), `wip`.
    TERMINATORS = ("qed", "wip")

    def consume_terminator(self) -> str:
        token = self.current
        if token.kind == "KEYWORD" and token.value in self.TERMINATORS:
            self.advance()
            return token.value
        self.fail("qed or wip")

    def parse_proof_steps(self) -> list[Any]:
        # Proof steps up to (but not consuming) the terminating `qed`/`wip`.
        # Shared by the top-level proof body and each named `case` block.
        steps: list[Any] = []
        while not (self.current.kind == "KEYWORD" and self.current.value in self.TERMINATORS):
            steps.append(self.parse_proof_step())
        return steps

    def parse_proof_step(self) -> Any:
        # Simple step:  goal <state> by <rewrite|assumption> therefore <state | done> ;
        # Apply step:   goal <state> by apply <call> <cases> therefore <state | done> <qed|wip> ;
        # An apply is a subproof, so its terminator follows the `therefore`.
        self.consume_keyword("goal")
        goal = self.parse_prop()
        self.consume_keyword("by")
        if self.current.kind == "KEYWORD" and self.current.value == "apply":
            tactic = self.parse_apply()
            self.consume_keyword("therefore")
            result = self.parse_proof_result()
            tactic.data["terminator"] = self.consume_terminator()
            self.consume(";")
        else:
            tactic = self.parse_tactic()
            self.consume_keyword("therefore")
            result = self.parse_proof_result()
            self.consume(";")
        return node("proof_step", goal=goal, tactic=tactic, result=result)

    def parse_proof_result(self) -> Any:
        if self.current.kind == "KEYWORD" and self.current.value == "done":
            self.advance()
            return node("done")
        return self.parse_prop()

    def parse_tactic(self) -> Any:
        token = self.current
        if token.kind == "KEYWORD" and token.value == "rewrite":
            return self.parse_rewrite()
        if token.kind == "KEYWORD" and token.value == "wip":
            self.advance()
            return node("wip_tactic")
        self.fail("tactic (rewrite or wip)")

    def parse_rewrite(self) -> Any:
        # rewrite > theorem(args) with ( lhs := rhs )   (or `<` for right-to-left)
        self.consume_keyword("rewrite")
        if self.match(">"):
            direction = ">"
        elif self.match("<"):
            direction = "<"
        else:
            self.fail("rewrite direction (> or <)")
        theorem = self.parse_theorem_instance()
        self.consume_keyword("with")
        self.consume("(")
        lhs = self.parse_expr()
        self.consume(":=")
        rhs = self.parse_expr()
        self.consume(")")
        return node("rewrite", direction=direction, theorem=theorem, lhs=lhs, rhs=rhs)

    def parse_theorem_instance(self) -> Any:
        # name(args) or a bare name (e.g. a local assumption `ih`).
        name = self.parse_rule_name("theorem name")
        args: list[Any] = []
        applied = False
        if self.current.value == "(":
            self.advance()
            args = self.parse_expr_list(")")
            applied = True
        return node("theorem_instance", name=name, args=args, applied=applied)

    def parse_apply(self) -> Any:
        # apply rule(args) case name proof_step* qed; ...
        # The application and its cases; the subproof terminator (qed/wip)
        # follows the step's `therefore` and is attached by parse_proof_step. A
        # zero-premise rule has no cases.
        self.consume_keyword("apply")
        name = self.parse_rule_name("rule name")
        self.consume("(")
        args = self.parse_expr_list(")")
        cases: list[Any] = []
        while self.current.kind == "KEYWORD" and self.current.value == "case":
            cases.append(self.parse_case())
        return node("apply", rule=name, args=args, cases=cases, terminator=None)

    def parse_case(self) -> Any:
        # case name proof_step* (qed|wip);   (matched to premises by name)
        self.consume_keyword("case")
        name = self.consume_ident("case name")
        steps = self.parse_proof_steps()
        terminator = self.consume_terminator()
        self.consume(";")
        return node("case", name=name, steps=steps, terminator=terminator)

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
            parts = [self.consume_ident()]
            while self.match("::"):
                parts.append(self.consume_ident("type name"))
            if len(parts) == 1 and parts[0] == "Seq" and self.match("["):
                item = self.parse_type_expr()
                self.consume("]")
                return node("type_sequence", item=item)
            args: list[Any] = []
            if self.match("["):
                args.append(self.parse_type_expr())
                while self.match(","):
                    args.append(self.parse_type_expr())
                self.consume("]")
            return node("type_name", module=parts[:-1], name=parts[-1], args=args)
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
        if token.value == "¬":
            self.advance()
            return node("unary", op=token.value, value=self.parse_binary(PRECEDENCE["."]))
        if token.value == "λ":
            # λ (a : A, b : B) => body  (ASCII: fun (...) => body). The body
            # extends greedily to the right, like if/let.
            self.advance()
            binders = self.parse_binder_list()
            self.consume("=>")
            body = self.parse_expr()
            return node("lambda", binders=binders, body=body)
        if token.value in {"∀", "∃"}:
            # ∀ (a : A, b b' : B) st body  /  ∃ (...) st body
            self.advance()
            binders = self.parse_binder_list()
            self.consume_keyword("st")
            body = self.parse_expr()
            kind = "forall" if token.value == "∀" else "exists"
            return node(kind, binders=binders, body=body)
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
            parts = [self.consume_ident()]
            while self.match("::"):
                parts.append(self.consume_ident("qualified name"))
            if len(parts) > 1:
                return node("qualified", module=parts[:-1], name=parts[-1])
            return node("identifier", name=parts[0])
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
