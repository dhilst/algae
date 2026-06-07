"""Type checker for .alg modules.

Bottom-up synthesis over the parsed AST. Ops may be overloaded by signature
and are resolved by argument types. Compatibility is structural equality
plus sum injection (a term of type T is acceptable where a sum containing T
is expected). Numeric sorts are strict: ℕ, ℤ, and ℝ are distinct types.
Narrowing a sum to one of its summands requires an explicit cast op declared
by the spec, conventionally `op cast : T | Error → T;`.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .ast import AxiomDecl, LetDecl, Module, Node, OpDecl, SortDecl, VarDecl, node
from .format import Formatter

NUMERIC = {"ℕ": 0, "ℤ": 1, "ℝ": 2}
BOOL = node("type_builtin", name="𝔹")
NAT = node("type_builtin", name="ℕ")
INT = node("type_builtin", name="ℤ")
STRING = node("type_builtin", name="String")  # internal; string literals only
COMPARISONS = {"=", "≠", "<", "≤", ">", "≥"}
ORDERINGS = {"<", "≤", ">", "≥"}
BOOL_OPS = {"∧", "∨", "⟹", "⟺"}
ARITHMETIC = {"+", "-", "*", "/", "×"}

_render = Formatter().type_expr


@dataclass(slots=True)
class TypeIssue:
    line: int
    message: str


def same_type(a: Node, b: Node) -> bool:
    if a.kind != b.kind:
        return False
    if a.kind in ("type_name", "type_builtin"):
        return a.data["name"] == b.data["name"]
    if a.kind == "type_unit":
        return True
    if a.kind == "type_sequence":
        return same_type(a.data["item"], b.data["item"])
    if a.kind == "type_function":
        return (
            a.data.get("partial", False) == b.data.get("partial", False)
            and same_type(a.data["left"], b.data["left"])
            and same_type(a.data["right"], b.data["right"])
        )
    if a.kind in ("type_product", "type_sum"):
        items_a, items_b = a.data["items"], b.data["items"]
        return len(items_a) == len(items_b) and all(
            same_type(x, y) for x, y in zip(items_a, items_b)
        )
    return False


def compatible(actual: Node, expected: Node) -> bool:
    """True when a term of type `actual` is acceptable where `expected` is required."""
    if same_type(actual, expected):
        return True
    if actual.kind == "type_sum" and expected.kind == "type_sum":
        return all(
            any(compatible(item, target) for target in expected.data["items"])
            for item in actual.data["items"]
        )
    if expected.kind == "type_sum":
        return any(compatible(actual, item) for item in expected.data["items"])
    if actual.kind == "type_product" and expected.kind == "type_product":
        items_a, items_b = actual.data["items"], expected.data["items"]
        return len(items_a) == len(items_b) and all(
            compatible(x, y) for x, y in zip(items_a, items_b)
        )
    return False


def is_numeric(t: Node) -> bool:
    return t.kind == "type_builtin" and t.data["name"] in NUMERIC


def widest(a: Node, b: Node) -> Node:
    return a if NUMERIC[a.data["name"]] >= NUMERIC[b.data["name"]] else b


class Checker:
    def __init__(self, module: Module) -> None:
        self.module = module
        self.issues: list[TypeIssue] = []
        self.line = 0
        self.sorts: set[str] = set()
        self.enum_values: dict[str, Node] = {}
        self.ops: dict[str, list[tuple[list[Node], Node]]] = {}
        self.vars: dict[str, Node] = {}
        self.lets: dict[str, Node] = {}

    def issue(self, message: str) -> None:
        self.issues.append(TypeIssue(self.line, message))

    def check(self) -> list[TypeIssue]:
        for decl in self.module.declarations:
            if isinstance(decl, SortDecl):
                self.line = decl.line
                self.collect_sort(decl)
        for decl in self.module.declarations:
            self.line = decl.line
            if isinstance(decl, OpDecl):
                self.collect_op(decl)
            elif isinstance(decl, VarDecl):
                self.collect_var(decl)
        # LemmaDecls are intentionally ignored: lemmas and their proofs are
        # parsed and stored only; checking them is a future phase.
        axiom_names: set[str] = set()
        for decl in self.module.declarations:
            self.line = decl.line
            if isinstance(decl, LetDecl):
                bound = self.synth(decl.expr, {})
                if bound is not None:
                    self.lets[decl.name] = bound
            elif isinstance(decl, AxiomDecl):
                if decl.name is not None:
                    if decl.name in axiom_names:
                        self.issue(f"duplicate axiom name {decl.name}")
                    axiom_names.add(decl.name)
                body = self.synth(decl.expr, {})
                if body is not None and not compatible(body, BOOL):
                    self.issue(f"axiom must be a boolean expression, got {_render(body)}")
        return self.issues

    # Declarations -----------------------------------------------------------

    def collect_sort(self, decl: SortDecl) -> None:
        for name in decl.names:
            if name in self.sorts:
                self.issue(f"duplicate sort {name}")
            self.sorts.add(name)
        if decl.values is not None:
            sort_type = node("type_name", name=decl.names[0])
            for value in decl.values:
                if value in self.enum_values:
                    self.issue(f"duplicate enum value {value}")
                self.enum_values[value] = sort_type

    def collect_op(self, decl: OpDecl) -> None:
        for item in decl.domain:
            self.check_type(item)
        self.check_type(decl.codomain)
        signatures = self.ops.setdefault(decl.name, [])
        for domain, _ in signatures:
            if len(domain) == len(decl.domain) and all(
                same_type(a, b) for a, b in zip(domain, decl.domain)
            ):
                self.issue(f"duplicate signature for op {decl.name}")
        signatures.append((decl.domain, decl.codomain))

    def collect_var(self, decl: VarDecl) -> None:
        self.check_type(decl.sort)
        for name in decl.names:
            if name in self.vars:
                self.issue(f"duplicate var {name}")
            self.vars[name] = decl.sort

    def check_type(self, t: Node) -> None:
        if t.kind == "type_name":
            if t.data["name"] not in self.sorts:
                self.issue(f"unknown sort {t.data['name']}")
        elif t.kind == "type_sequence":
            self.check_type(t.data["item"])
        elif t.kind == "type_function":
            self.check_type(t.data["left"])
            self.check_type(t.data["right"])
        elif t.kind in ("type_product", "type_sum"):
            for item in t.data["items"]:
                self.check_type(item)

    # Expressions ------------------------------------------------------------

    def synth(self, expr: Any, env: dict[str, Node]) -> Node | None:
        if not isinstance(expr, Node):
            self.issue(f"unsupported expression: {expr!r}")
            return None
        kind = expr.kind
        data = expr.data
        if kind == "identifier":
            return self.synth_identifier(data["name"], env)
        if kind == "number":
            return NAT
        if kind == "string":
            return STRING
        if kind in ("bool", "bool_symbol"):
            return BOOL
        if kind == "builtin_set":
            self.issue(f"{data['name']} is a sort, not a term")
            return None
        if kind == "unit":
            return node("type_unit")
        if kind == "tuple":
            items = [self.synth(item, env) for item in data["items"]]
            if any(item is None for item in items):
                return None
            return node("type_product", items=items)
        if kind == "call":
            return self.synth_call(data["function"], data["args"], env)
        if kind == "prime":
            return self.synth(data["value"], env)
        if kind == "unary":
            return self.synth_unary(data["op"], data["value"], env)
        if kind == "binary":
            return self.synth_binary(data["op"], data["left"], data["right"], env)
        if kind == "if":
            return self.synth_if(data, env)
        if kind == "let":
            value = self.synth(data["value"], env)
            if value is None:
                return None
            return self.synth(data["body"], {**env, data["name"]: value})
        if kind == "let_tuple":
            return self.synth_let_tuple(data, env)
        self.issue(f"unsupported expression kind {kind}")
        return None

    def synth_identifier(self, name: str, env: dict[str, Node]) -> Node | None:
        if name in env:
            return env[name]
        if name in self.vars:
            return self.vars[name]
        if name in self.enum_values:
            return self.enum_values[name]
        if name in self.lets:
            return self.lets[name]
        if name in self.ops:
            signatures = self.ops[name]
            if len(signatures) != 1:
                self.issue(f"ambiguous reference to overloaded op {name}")
                return None
            domain, codomain = signatures[0]
            if not domain:
                self.issue(f"nullary op {name} must be called: {name}()")
                return None
            left = domain[0] if len(domain) == 1 else node("type_product", items=domain)
            return node("type_function", left=left, right=codomain)
        self.issue(f"undeclared identifier {name}")
        return None

    def synth_call(self, function: Any, args: list[Any], env: dict[str, Node]) -> Node | None:
        arg_types = [self.synth(arg, env) for arg in args]
        if any(arg is None for arg in arg_types):
            return None
        if (
            isinstance(function, Node)
            and function.kind == "identifier"
            and function.data["name"] not in env
            and function.data["name"] not in self.vars
            and function.data["name"] not in self.lets
            and function.data["name"] in self.ops
        ):
            return self.resolve_op(function.data["name"], arg_types)
        callee = self.synth(function, env)
        if callee is None:
            return None
        if callee.kind != "type_function":
            self.issue(f"{_render(callee)} is not callable")
            return None
        param = callee.data["left"]
        if len(args) == 1:
            if not compatible(arg_types[0], param):
                self.issue(f"argument of type {_render(arg_types[0])} does not match {_render(param)}")
                return None
        elif param.kind == "type_product" and len(param.data["items"]) == len(args):
            for actual, expected in zip(arg_types, param.data["items"]):
                if not compatible(actual, expected):
                    self.issue(f"argument of type {_render(actual)} does not match {_render(expected)}")
                    return None
        else:
            self.issue(f"expected arguments matching {_render(param)}, got {len(args)}")
            return None
        return callee.data["right"]

    def resolve_op(self, name: str, arg_types: list[Node]) -> Node | None:
        signatures = self.ops[name]
        matches = [
            (domain, codomain)
            for domain, codomain in signatures
            if len(domain) == len(arg_types)
            and all(compatible(actual, expected) for actual, expected in zip(arg_types, domain))
        ]
        if len(matches) == 1:
            return matches[0][1]
        if not matches:
            rendered = ", ".join(_render(t) for t in arg_types)
            self.issue(f"no signature of {name} matches ({rendered})")
            return None
        exact = [
            (domain, codomain)
            for domain, codomain in matches
            if all(same_type(actual, expected) for actual, expected in zip(arg_types, domain))
        ]
        if len(exact) == 1:
            return exact[0][1]
        rendered = ", ".join(_render(t) for t in arg_types)
        self.issue(f"ambiguous call {name}({rendered})")
        return None

    def synth_unary(self, op: str, value: Any, env: dict[str, Node]) -> Node | None:
        operand = self.synth(value, env)
        if operand is None:
            return None
        if op == "¬":
            if not compatible(operand, BOOL):
                self.issue(f"¬ requires 𝔹, got {_render(operand)}")
                return None
            return BOOL
        if not is_numeric(operand):
            self.issue(f"unary - requires a numeric type, got {_render(operand)}")
            return None
        # Negation leaves the naturals: -n is an integer, not a natural.
        return INT if same_type(operand, NAT) else operand

    def synth_binary(self, op: str, left: Any, right: Any, env: dict[str, Node]) -> Node | None:
        if op in (".", "▷"):
            return self.synth_application_sugar(op, left, right, env)
        left_t = self.synth(left, env)
        right_t = self.synth(right, env)
        if left_t is None or right_t is None:
            return None
        if op in COMPARISONS:
            if op in ORDERINGS:
                if not (is_numeric(left_t) and is_numeric(right_t)):
                    self.issue(f"{op} requires numeric operands, got {_render(left_t)} and {_render(right_t)}")
                    return None
            elif not (compatible(left_t, right_t) or compatible(right_t, left_t)):
                self.issue(f"cannot equate {_render(left_t)} with {_render(right_t)}")
                return None
            return BOOL
        if op in BOOL_OPS:
            for operand in (left_t, right_t):
                if not compatible(operand, BOOL):
                    self.issue(f"{op} requires 𝔹 operands, got {_render(operand)}")
                    return None
            return BOOL
        if op in ARITHMETIC:
            if not (is_numeric(left_t) and is_numeric(right_t)):
                self.issue(f"{op} requires numeric operands, got {_render(left_t)} and {_render(right_t)}")
                return None
            result = widest(left_t, right_t)
            # Subtraction is not closed over the naturals: ℕ - ℕ yields ℤ.
            if op == "-" and same_type(result, NAT):
                return INT
            return result
        if op == "++":
            if (
                left_t.kind == "type_sequence"
                and right_t.kind == "type_sequence"
                and (compatible(left_t, right_t) or compatible(right_t, left_t))
            ):
                return left_t
            self.issue(f"++ requires matching Seq operands, got {_render(left_t)} and {_render(right_t)}")
            return None
        self.issue(f"unsupported operator {op}")
        return None

    def synth_application_sugar(self, op: str, left: Any, right: Any, env: dict[str, Node]) -> Node | None:
        # x.f(a) reads as f(x, a); x ▷ f(a) reads as f(a, x); bare names as f(x).
        if isinstance(right, Node) and right.kind == "call":
            args = right.data["args"]
            new_args = [left, *args] if op == "." else [*args, left]
            return self.synth_call(right.data["function"], new_args, env)
        if isinstance(right, Node) and right.kind == "identifier":
            return self.synth_call(right, [left], env)
        self.issue(f"right operand of {op} must be a call or operation name")
        return None

    def synth_if(self, data: dict[str, Any], env: dict[str, Node]) -> Node | None:
        condition = self.synth(data["condition"], env)
        if condition is not None and not compatible(condition, BOOL):
            self.issue(f"if condition must be 𝔹, got {_render(condition)}")
        then_t = self.synth(data["then"], env)
        else_t = self.synth(data["otherwise"], env)
        if then_t is None or else_t is None:
            return None
        if compatible(then_t, else_t):
            return else_t
        if compatible(else_t, then_t):
            return then_t
        self.issue(f"if branches disagree: {_render(then_t)} versus {_render(else_t)}")
        return None

    def synth_let_tuple(self, data: dict[str, Any], env: dict[str, Node]) -> Node | None:
        value = self.synth(data["value"], env)
        if value is None:
            return None
        binders = data["binders"]
        if value.kind == "type_sum":
            self.issue(f"cannot destructure sum type {_render(value)}; the pattern requires a product")
            return None
        if value.kind != "type_product":
            self.issue(f"cannot destructure non-product type {_render(value)}")
            return None
        items = value.data["items"]
        if len(items) != len(binders):
            self.issue(
                f"pattern has {len(binders)} binders but {_render(value)} has {len(items)} components"
            )
            return None
        named = [binder for binder in binders if binder != "_"]
        duplicates = sorted({binder for binder in named if named.count(binder) > 1})
        if duplicates:
            self.issue(f"duplicate binder {duplicates[0]} in destructuring pattern")
            return None
        scope = dict(env)
        for binder, item in zip(binders, items):
            if binder != "_":
                scope[binder] = item
        return self.synth(data["body"], scope)


def check_module(module: Module) -> list[TypeIssue]:
    return Checker(module).check()
