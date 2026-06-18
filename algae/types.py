"""Type checker for .alg modules.

Bottom-up synthesis over the parsed AST. Ops may be overloaded by signature
and are resolved by argument types. Compatibility is structural equality
plus sum injection (a term of type T is acceptable where a sum containing T
is expected). Sorts and their parameters are user-declared with explicit kinds
(`Sort`, `Sort → Sort`, …); there are no built-in value sorts. `𝔹` and `Prop`
are the only built-in logical types. Proofs (`goal/by/therefore/done`,
`rewrite`, `apply`, and include obligations) are parsed and structurally
checked but never discharged.
"""

from __future__ import annotations

from dataclasses import dataclass
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
    node,
)
from .format import Formatter

BOOL = node("type_builtin", name="𝔹")
PROP = node("type_builtin", name="Prop")  # the type of propositions, for rule predicates
SORT = node("type_builtin", name="Sort")  # the kind of sorts
COMPARISONS = {"=", "≠"}
BOOL_OPS = {"∧", "∨", "⟹", "⟺"}

_render = Formatter().type_expr
_render_expr = Formatter().expr


@dataclass(slots=True)
class TypeIssue:
    line: int
    message: str


def same_type(a: Node, b: Node) -> bool:
    if a.kind != b.kind:
        return False
    if a.kind == "type_builtin":
        return a.data["name"] == b.data["name"]
    if a.kind == "type_name":
        args_a, args_b = a.data.get("args", []), b.data.get("args", [])
        return (
            a.data.get("module", []) == b.data.get("module", [])
            and a.data["name"] == b.data["name"]
            and len(args_a) == len(args_b)
            and all(same_type(x, y) for x, y in zip(args_a, args_b))
        )
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


def is_prop_like(t: Node | None) -> bool:
    """A proposition is anything boolean (a concrete equation) or of type Prop
    (a predicate application like P(x))."""
    return t is not None and (compatible(t, BOOL) or same_type(t, PROP))


def is_kind(t: Any) -> bool:
    """True for a kind: Sort, or Sort → … → Sort (a sort constructor)."""
    cur = t
    while isinstance(cur, Node) and cur.kind == "type_function":
        cur = cur.data["right"]
    return isinstance(cur, Node) and cur.kind == "type_builtin" and cur.data["name"] == "Sort"


def kind_arity(kind: Any) -> int:
    n = 0
    cur = kind
    while isinstance(cur, Node) and cur.kind == "type_function":
        n += 1
        cur = cur.data["right"]
    return n


class Checker:
    def __init__(self, module: Module, loader: Any = None) -> None:
        self.module = module
        self.loader = loader
        self.issues: list[TypeIssue] = []
        self.line = 0
        self.sorts: set[str] = set()
        self.sort_arity: dict[str, int] = {}  # sort/param name → kind arity
        self.params: dict[str, Node] = {}  # module parameter name → kind
        self.local_type_vars: dict[str, int] = {}  # binder-bound type vars → arity
        self.ops: dict[str, list[tuple[list[Node], Node]]] = {}
        self.lets: dict[str, Node] = {}
        self.rules: dict[str, RuleDecl] = {}
        # eq, prop, lemma, and rule names share one namespace.
        self.proof_names: set[str] = set()
        self.aliases: dict[str, list[str]] = {}  # alias name → full module path
        self.included: set[str] = set()  # joined paths of included modules
        # id(IncludeDecl) → (prop_map, namespace, own_sorts, params, type_mapping)
        self.include_obligations: dict[int, tuple] = {}

    def expand_alias(self, module: list[str]) -> list[str]:
        # `alias bar = foo::bar;` lets `bar::x` stand for `foo::bar::x`.
        if module and module[0] in self.aliases:
            return self.aliases[module[0]] + module[1:]
        return list(module)

    def issue(self, message: str) -> None:
        self.issues.append(TypeIssue(self.line, message))

    def check(self) -> list[TypeIssue]:
        for decl in self.module.declarations:
            self.line = decl.line
            if isinstance(decl, SortDecl):
                self.collect_sort(decl)
            elif isinstance(decl, ParamDecl):
                self.collect_param(decl)
        # Module pre-pass: aliases, then includes (which register namespaced
        # symbols), then opens (which expose included names unqualified). Run
        # after local sorts so `with (T := LocalSort)` resolves.
        for decl in self.module.declarations:
            if isinstance(decl, AliasDecl):
                self.line = decl.line
                self.aliases[decl.alias] = list(decl.path)
        for decl in self.module.declarations:
            if isinstance(decl, IncludeDecl):
                self.line = decl.line
                self.register_include(decl)
        for decl in self.module.declarations:
            if isinstance(decl, OpenDecl):
                self.line = decl.line
                self.register_open(decl)
        for decl in self.module.declarations:
            self.line = decl.line
            if isinstance(decl, OpDecl):
                self.collect_op(decl)
            elif isinstance(decl, RuleDecl):
                # Collect rules up front so `apply` can resolve forward references.
                self.rules.setdefault(decl.name, decl)
        # Final pass: check let/eq/prop bodies, rule premises and conclusions,
        # lemma propositions and their proofs, and include obligations. Proof
        # steps are checked structurally; rewrites are never discharged.
        for decl in self.module.declarations:
            self.line = decl.line
            if isinstance(decl, LetDecl):
                bound = self.synth(decl.expr, {})
                if bound is not None:
                    self.lets[decl.name] = bound
            elif isinstance(decl, EqDecl):
                self.register_proof_name(decl.name, "eq")
                self.local_type_vars = {}
                self.check_prop(decl.expr, self.binder_env(decl.params))
            elif isinstance(decl, PropDecl):
                self.register_proof_name(decl.name, "prop")
                self.local_type_vars = {}
                self.check_prop(decl.expr, self.binder_env(decl.params))
            elif isinstance(decl, RuleDecl):
                self.register_proof_name(decl.name, "rule")
                self.check_rule(decl)
            elif isinstance(decl, LemmaDecl):
                self.register_proof_name(decl.name, "lemma")
                self.local_type_vars = {}
                env = self.binder_env(decl.params)
                self.check_prop(decl.expr, env)
                if decl.proof is not None:
                    self.check_proof(decl.proof, env)
            elif isinstance(decl, IncludeDecl):
                self.check_include_obligations(decl)
        return self.issues

    def binder_env(self, params: list[Any]) -> dict[str, Node]:
        # Build a local scope from explicit binders. A binder whose type is a
        # kind (Sort, Sort → Sort, …) introduces a local type variable; any
        # other binder introduces a value variable. Validates each type.
        env: dict[str, Node] = {}
        for name, btype in params:
            if is_kind(btype):
                self.local_type_vars[name] = kind_arity(btype)
                continue
            self.check_type(btype)
            if name in env:
                self.issue(f"duplicate binder {name}")
            env[name] = btype
        return env

    def register_proof_name(self, name: str | None, kind: str) -> None:
        # eq, prop, lemma, and rule names share one namespace; the message names
        # the kind of the colliding declaration.
        if name is None:
            return
        if name in self.proof_names:
            self.issue(f"duplicate {kind} name {name}")
        self.proof_names.add(name)

    # Modules ----------------------------------------------------------------

    def register_include(self, decl: IncludeDecl) -> None:
        if self.loader is None:
            self.issue("include requires an alg-project.json project root")
            return
        prefix = "::".join(decl.path)
        if not self.loader.begin_check(prefix):
            self.issue(f"circular include {prefix}")
            return
        try:
            self._register_include(decl, prefix)
        finally:
            self.loader.end_check(prefix)

    def _register_include(self, decl: IncludeDecl, prefix: str) -> None:
        try:
            module = self.loader.load(decl.path)
        except Exception as exc:  # ModuleError and IO/parse errors
            self.issue(str(exc))
            return
        # Validate the included module on its own terms (its includes resolved
        # through the same loader) so broken modules are reported once.
        sub = Checker(module, loader=self.loader)
        sub_issues = sub.check()
        if sub_issues:
            self.issue(f"included module {prefix} has errors: {sub_issues[0].message}")
        sort_decls = [d for d in module.declarations if isinstance(d, SortDecl)]
        param_decls = [d for d in module.declarations if isinstance(d, ParamDecl)]
        op_decls = [d for d in module.declarations if isinstance(d, OpDecl)]
        prop_decls = [d for d in module.declarations if isinstance(d, PropDecl)]
        own_sorts = {d.name for d in sort_decls}
        params = {d.name: d.kind_expr for d in param_decls}
        # `with (...)` substitutions: a param LHS takes a type RHS; an op LHS
        # takes an op-name RHS (validated structurally only).
        type_mapping: dict[str, Node] = {}
        op_names = {d.name for d in op_decls}
        for pname, value in decl.bindings:
            if pname in params:
                self.check_type(value)
                type_mapping[pname] = value
            elif pname in op_names:
                pass  # op substitution; not discharged in structure-only mode
            else:
                self.issue(f"{pname} is not a parameter or op of module {prefix}")
        # Unbound params stay abstract: register them as namespaced opaque sorts.
        for d in param_decls:
            if d.name not in type_mapping:
                key = f"{prefix}::{d.name}"
                self.sorts.add(key)
                self.sort_arity[key] = kind_arity(d.kind_expr)

        def imp(t: Node) -> Node:
            return self.import_type(t, decl.path, own_sorts, params, type_mapping)

        for d in sort_decls:
            key = f"{prefix}::{d.name}"
            self.sorts.add(key)
            self.sort_arity[key] = kind_arity(d.kind_expr)
        for d in op_decls:
            domain = [imp(t) for t in d.domain]
            self.ops.setdefault(f"{prefix}::{d.name}", []).append((domain, imp(d.codomain)))
        self.included.add(prefix)
        prop_map = {d.name: d for d in prop_decls}
        self.include_obligations[id(decl)] = (
            prop_map, list(decl.path), own_sorts, params, type_mapping
        )

    def register_open(self, decl: OpenDecl) -> None:
        prefix = "::".join(decl.path)
        if prefix not in self.included:
            self.issue(f"open of un-included module {prefix}")
            return
        for name in decl.names:
            key = f"{prefix}::{name}"
            if key in self.ops:
                if name in self.ops or name in self.sorts:
                    self.issue(f"open name {name} collides with an existing name")
                    continue
                self.ops.setdefault(name, []).extend(self.ops[key])
            elif key in self.sorts:
                self.sorts.add(name)
                self.sort_arity[name] = self.sort_arity.get(key, 0)
            else:
                self.issue(f"name {name} is not exported by module {prefix}")

    def import_type(
        self,
        t: Node,
        namespace: list[str],
        own_sorts: set[str],
        params: dict[str, Node],
        mapping: dict[str, Node],
    ) -> Node:
        # Rewrite a type from an included module into the importer's view:
        # substitute `with` bindings for parameters and qualify references to
        # the module's own sorts and unbound parameters.
        if not isinstance(t, Node):
            return t
        kind, data = t.kind, t.data
        if kind == "type_name":
            module = data.get("module", [])
            args = [self.import_type(a, namespace, own_sorts, params, mapping) for a in data.get("args", [])]
            if module:
                return node("type_name", module=module, name=data["name"], args=args)
            name = data["name"]
            if name in mapping and not args:
                return mapping[name]
            if name in params or name in own_sorts:
                return node("type_name", module=list(namespace), name=name, args=args)
            return node("type_name", module=[], name=name, args=args)
        if kind in ("type_builtin", "type_unit"):
            return t
        if kind == "type_sequence":
            return node("type_sequence", item=self.import_type(data["item"], namespace, own_sorts, params, mapping))
        if kind == "type_function":
            return node(
                "type_function",
                left=self.import_type(data["left"], namespace, own_sorts, params, mapping),
                right=self.import_type(data["right"], namespace, own_sorts, params, mapping),
                partial=data.get("partial", False),
            )
        if kind in ("type_product", "type_sum"):
            return node(kind, items=[self.import_type(i, namespace, own_sorts, params, mapping) for i in data["items"]])
        return t

    # Declarations -----------------------------------------------------------

    def collect_sort(self, decl: SortDecl) -> None:
        if decl.name in self.sorts:
            self.issue(f"duplicate sort {decl.name}")
        self.check_kind(decl.kind_expr)
        self.sorts.add(decl.name)
        self.sort_arity[decl.name] = kind_arity(decl.kind_expr)

    def collect_param(self, decl: ParamDecl) -> None:
        if decl.name in self.sorts:
            self.issue(f"duplicate sort or param {decl.name}")
        self.check_kind(decl.kind_expr)
        self.sorts.add(decl.name)
        self.sort_arity[decl.name] = kind_arity(decl.kind_expr)
        self.params[decl.name] = decl.kind_expr

    def check_kind(self, kind: Any) -> None:
        cur = kind
        while isinstance(cur, Node) and cur.kind == "type_function":
            left = cur.data["left"]
            if not (left.kind == "type_builtin" and left.data["name"] == "Sort"):
                self.issue("kind arguments must be Sort")
            cur = cur.data["right"]
        if not (isinstance(cur, Node) and cur.kind == "type_builtin" and cur.data["name"] == "Sort"):
            self.issue("a kind must be Sort or Sort → … → Sort")

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

    def check_type(self, t: Node) -> None:
        if t.kind == "type_name":
            module = t.data.get("module", [])
            args = t.data.get("args", [])
            for arg in args:
                self.check_type(arg)
            if module:
                key = "::".join(self.expand_alias(module) + [t.data["name"]])
                if key not in self.sorts:
                    self.issue(f"unknown sort {key}")
                elif len(args) != self.sort_arity.get(key, 0):
                    self.issue(
                        f"sort {key} takes {self.sort_arity.get(key, 0)} type argument(s), got {len(args)}"
                    )
                return
            name = t.data["name"]
            if name in self.local_type_vars:
                expected = self.local_type_vars[name]
                if len(args) != expected:
                    self.issue(f"type variable {name} takes {expected} type argument(s), got {len(args)}")
                return
            if name not in self.sorts:
                self.issue(f"unknown sort {name}")
            elif len(args) != self.sort_arity.get(name, 0):
                self.issue(
                    f"sort {name} takes {self.sort_arity.get(name, 0)} type argument(s), got {len(args)}"
                )
        elif t.kind == "type_sequence":
            self.check_type(t.data["item"])
        elif t.kind == "type_function":
            self.check_type(t.data["left"])
            self.check_type(t.data["right"])
        elif t.kind in ("type_product", "type_sum"):
            for item in t.data["items"]:
                self.check_type(item)
        # type_builtin (𝔹, Prop, Sort) and type_unit need no checking.

    # Expressions ------------------------------------------------------------

    def synth(self, expr: Any, env: dict[str, Node]) -> Node | None:
        if not isinstance(expr, Node):
            self.issue(f"unsupported expression: {expr!r}")
            return None
        kind = expr.kind
        data = expr.data
        if kind == "identifier":
            return self.synth_identifier(data["name"], env)
        if kind == "qualified":
            return self.synth_qualified(data["module"], data["name"])
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
        if kind == "lambda":
            scope = dict(env)
            for name, btype in data["binders"]:
                self.check_type(btype)
                scope[name] = btype
            body = self.synth(data["body"], scope)
            if body is None:
                return None
            # A predicate (boolean/Prop body) has codomain Prop, so it matches a
            # `T → Prop` rule parameter; otherwise the body's own type. Several
            # binders take a product domain (callable as `f(a, b)`), matching the
            # op convention.
            result = PROP if is_prop_like(body) else body
            binders = data["binders"]
            if not binders:
                return result
            left = binders[0][1] if len(binders) == 1 else node("type_product", items=[t for _, t in binders])
            return node("type_function", left=left, right=result, partial=False)
        if kind in ("forall", "exists"):
            scope = dict(env)
            for name, btype in data["binders"]:
                self.check_type(btype)
                scope[name] = btype
            body = self.synth(data["body"], scope)
            if body is not None and not is_prop_like(body):
                self.issue(f"quantifier body must be a proposition, got {_render(body)}")
            return PROP
        self.issue(f"unsupported expression kind {kind}")
        return None

    def synth_identifier(self, name: str, env: dict[str, Node]) -> Node | None:
        if name in env:
            return env[name]
        if name in self.lets:
            return self.lets[name]
        if name in self.ops:
            return self.op_reference(name)
        self.issue(f"undeclared identifier {name}")
        return None

    def synth_qualified(self, module: list[str], name: str) -> Node | None:
        key = "::".join(self.expand_alias(module) + [name])
        if key in self.ops:
            return self.op_reference(key)
        self.issue(f"undeclared name {key}")
        return None

    def op_reference(self, key: str) -> Node | None:
        # The type of an op named (unapplied). A nullary op is a constant of its
        # codomain; any other op is referenced as its function type.
        signatures = self.ops[key]
        if len(signatures) != 1:
            self.issue(f"ambiguous reference to overloaded op {key}")
            return None
        domain, codomain = signatures[0]
        if not domain:
            return codomain
        left = domain[0] if len(domain) == 1 else node("type_product", items=domain)
        return node("type_function", left=left, right=codomain)

    def call_op_key(self, function: Any, env: dict[str, Node]) -> str | None:
        # The ops-table key a call targets, or None if `function` is not a
        # (non-shadowed) op name. Handles bare and qualified names.
        if not isinstance(function, Node):
            return None
        if function.kind == "identifier":
            name = function.data["name"]
            if name in env or name in self.lets:
                return None
            return name if name in self.ops else None
        if function.kind == "qualified":
            key = "::".join(self.expand_alias(function.data["module"]) + [function.data["name"]])
            return key if key in self.ops else None
        return None

    def synth_call(self, function: Any, args: list[Any], env: dict[str, Node]) -> Node | None:
        arg_types = [self.synth(arg, env) for arg in args]
        if any(arg is None for arg in arg_types):
            return None
        op_key = self.call_op_key(function, env)
        if op_key is not None:
            return self.resolve_op(op_key, arg_types)
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
            if not is_prop_like(operand):
                self.issue(f"¬ requires a proposition (𝔹 or Prop), got {_render(operand)}")
                return None
            return PROP if same_type(operand, PROP) else BOOL
        self.issue(f"unsupported operator {op}")
        return None

    def synth_binary(self, op: str, left: Any, right: Any, env: dict[str, Node]) -> Node | None:
        if op in (".", "▷"):
            return self.synth_application_sugar(op, left, right, env)
        left_t = self.synth(left, env)
        right_t = self.synth(right, env)
        if left_t is None or right_t is None:
            return None
        if op in COMPARISONS:
            if not (compatible(left_t, right_t) or compatible(right_t, left_t)):
                self.issue(f"cannot equate {_render(left_t)} with {_render(right_t)}")
                return None
            return BOOL
        if op in BOOL_OPS:
            for operand in (left_t, right_t):
                if not is_prop_like(operand):
                    self.issue(f"{op} requires proposition (𝔹 or Prop) operands, got {_render(operand)}")
                    return None
            return PROP if (same_type(left_t, PROP) or same_type(right_t, PROP)) else BOOL
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

    # Type arguments (sorts passed to a rule/theorem) ------------------------

    def expr_as_type(self, expr: Any) -> Node | None:
        # A type argument is written as an expression naming a sort.
        if isinstance(expr, Node):
            if expr.kind == "identifier":
                return node("type_name", module=[], name=expr.data["name"], args=[])
            if expr.kind == "qualified":
                return node("type_name", module=list(expr.data["module"]), name=expr.data["name"], args=[])
        return None

    def subst_type(self, t: Any, subst: dict[str, Node]) -> Any:
        if not isinstance(t, Node):
            return t
        if t.kind == "type_name" and not t.data.get("module") and not t.data.get("args") and t.data["name"] in subst:
            return subst[t.data["name"]]
        if t.kind == "type_name":
            return node(
                "type_name",
                module=t.data.get("module", []),
                name=t.data["name"],
                args=[self.subst_type(a, subst) for a in t.data.get("args", [])],
            )
        if t.kind == "type_function":
            return node(
                "type_function",
                left=self.subst_type(t.data["left"], subst),
                right=self.subst_type(t.data["right"], subst),
                partial=t.data.get("partial", False),
            )
        if t.kind in ("type_product", "type_sum"):
            return node(t.kind, items=[self.subst_type(i, subst) for i in t.data["items"]])
        if t.kind == "type_sequence":
            return node("type_sequence", item=self.subst_type(t.data["item"], subst))
        return t

    # Propositions, rules, and proofs ----------------------------------------

    def sequent_env(self, prop: Any, env: dict[str, Node]) -> dict[str, Node]:
        # Extend an environment with the typed context variables of a sequent.
        scope = dict(env)
        if isinstance(prop, Node) and prop.kind == "sequent":
            for entry in prop.data["assumptions"]:
                if entry.kind == "context_var":
                    self.check_type(entry.data["type"])
                    scope[entry.data["name"]] = entry.data["type"]
        return scope

    def check_prop(self, prop: Any, env: dict[str, Node]) -> None:
        # Every assumption and the goal must be a proposition (𝔹 or Prop). No
        # logical content is verified.
        if isinstance(prop, Node) and prop.kind == "sequent":
            scope = self.sequent_env(prop, env)
            for entry in prop.data["assumptions"]:
                if entry.kind == "context_var":
                    continue
                t = self.synth(entry.data["expr"], scope)
                if t is not None and not is_prop_like(t):
                    self.issue(f"assumption must be a proposition, got {_render(t)}")
            goal = self.synth(prop.data["goal"], scope)
            if goal is not None and not is_prop_like(goal):
                self.issue(f"goal must be a proposition, got {_render(goal)}")
        else:
            t = self.synth(prop, env)
            if t is not None and not is_prop_like(t):
                self.issue(f"proposition must be a 𝔹 or Prop expression, got {_render(t)}")

    def check_rule(self, decl: RuleDecl) -> None:
        self.local_type_vars = {}
        param_env = self.binder_env(decl.params)
        names = [p.data["name"] for p in decl.premises]
        duplicates = sorted({n for n in names if names.count(n) > 1})
        if duplicates:
            self.issue(f"duplicate rule premise case {duplicates[0]}")
        for premise in decl.premises:
            prop = premise.data["prop"]
            self.check_prop(prop, param_env)
            # Hypotheses are named only in proof cases; a rule premise must not
            # name its assumptions.
            for entry in self.premise_assumptions(prop):
                if entry.kind == "assumption" and entry.data["name"] is not None:
                    self.issue(
                        f"rule premise assumptions must be unnamed; found {entry.data['name']} :="
                    )
        self.check_prop(decl.conclusion, param_env)

    @staticmethod
    def premise_assumptions(prop: Any) -> list[Any]:
        if isinstance(prop, Node) and prop.kind == "sequent":
            return prop.data["assumptions"]
        return []

    def check_proof(self, proof: Any, env: dict[str, Node]) -> None:
        for step in proof.data["steps"]:
            self.check_proof_step(step, env)
        self.check_terminator(
            proof.data["terminator"], self._steps_use_wip(proof.data["steps"]), "this proof"
        )

    def check_proof_step(self, step: Node, env: dict[str, Node]) -> None:
        goal = step.data["goal"]
        self.check_prop(goal, env)
        step_env = self.sequent_env(goal, env)
        self.check_tactic(step.data["tactic"], step_env)
        result = step.data["result"]
        if not (isinstance(result, Node) and result.kind == "done"):
            self.check_prop(result, env)

    def check_tactic(self, tactic: Node, env: dict[str, Node]) -> None:
        if tactic.kind == "apply":
            self.check_apply(tactic, env)
        elif tactic.kind == "rewrite":
            self.check_rewrite(tactic, env)
        # `wip` is structural: the step's goal and result are checked by
        # check_proof_step; the tactic itself only marks the subproof as work in
        # progress, so it must be closed with `wip` (enforced by check_terminator).

    def _steps_use_wip(self, steps: list[Any]) -> bool:
        # True when a proof body is still work in progress — it uses the `wip`
        # tactic directly, or through an `apply` whose own cases do. This drives
        # the viral qed/wip rule: such a subproof must be closed with `wip`.
        for step in steps:
            tactic = step.data["tactic"]
            if tactic.kind == "wip_tactic":
                return True
            if tactic.kind == "apply" and self._apply_uses_wip(tactic):
                return True
        return False

    def _apply_uses_wip(self, apply: Node) -> bool:
        return any(self._steps_use_wip(c.data["steps"]) for c in apply.data["cases"])

    def check_terminator(self, terminator: str, is_wip: bool, what: str) -> None:
        if terminator == "qed" and is_wip:
            self.issue(f"{what} is work in progress (uses `wip`); close it with `wip`, not `qed`")

    def check_apply(self, tactic: Node, env: dict[str, Node]) -> None:
        name = tactic.data["rule"]
        cases = tactic.data["cases"]
        rule = self.rules.get(name)
        if rule is None:
            self.issue(f"unknown rule {name}")
        else:
            args = tactic.data["args"]
            if len(args) != len(rule.params):
                self.issue(f"rule {name} expects {len(rule.params)} argument(s), got {len(args)}")
            else:
                self.check_apply_args(rule, args, env)
            premise_names = sorted(p.data["name"] for p in rule.premises)
            case_names = sorted(c.data["name"] for c in cases)
            if case_names != premise_names:
                self.issue(
                    f"apply {name} requires cases {premise_names}, got {case_names}"
                )
        self.check_cases(cases, env)
        self.check_terminator(tactic.data["terminator"], self._apply_uses_wip(tactic), f"apply {name}")

    def check_apply_args(self, rule: RuleDecl, args: list[Any], env: dict[str, Node]) -> None:
        # Sort-kinded parameters take type arguments; substitute them into the
        # remaining value-parameter types before checking the value arguments.
        type_subst: dict[str, Node] = {}
        for (pname, ptype), arg in zip(rule.params, args):
            if is_kind(ptype):
                as_type = self.expr_as_type(arg)
                if as_type is None:
                    self.issue(f"argument for {pname} must name a sort")
                else:
                    self.check_type(as_type)
                    type_subst[pname] = as_type
        for (pname, ptype), arg in zip(rule.params, args):
            if is_kind(ptype):
                continue
            actual = self.synth(arg, env)
            if actual is None:
                continue
            expected = self.subst_type(ptype, type_subst)
            if not compatible(actual, expected):
                self.issue(
                    f"argument for {pname} has type {_render(actual)}, expected {_render(expected)}"
                )

    def check_rewrite(self, tactic: Node, env: dict[str, Node]) -> None:
        # Structure only: the rewrite is recorded but never discharged. The
        # theorem arguments and the replacement terms are type-checked best-effort.
        for arg in tactic.data["theorem"].data["args"]:
            self.synth(arg, env)
        self.synth(tactic.data["lhs"], env)
        self.synth(tactic.data["rhs"], env)

    def check_cases(self, cases: list[Any], env: dict[str, Node]) -> None:
        for case in cases:
            for step in case.data["steps"]:
                self.check_proof_step(step, env)
            self.check_terminator(
                case.data["terminator"],
                self._steps_use_wip(case.data["steps"]),
                f"case {case.data['name']}",
            )

    def check_include_obligations(self, decl: IncludeDecl) -> None:
        info = self.include_obligations.get(id(decl))
        if info is None:
            return
        prop_map, namespace, own_sorts, params, type_mapping = info
        prop_names = sorted(prop_map)
        case_names = sorted(c.data["name"] for c in decl.obligations)
        if case_names != prop_names:
            self.issue(
                f"include {'::'.join(decl.path)} requires obligation cases {prop_names}, got {case_names}"
            )
        for case in decl.obligations:
            prop = prop_map.get(case.data["name"])
            env: dict[str, Node] = {}
            if prop is not None:
                for pname, ptype in prop.params:
                    env[pname] = self.import_type(ptype, namespace, own_sorts, params, type_mapping)
            for step in case.data["steps"]:
                self.check_proof_step(step, env)
            self.check_terminator(
                case.data["terminator"],
                self._steps_use_wip(case.data["steps"]),
                f"case {case.data['name']}",
            )
        # The `props` block is itself a subproof: it is work in progress if any
        # obligation case is `wip`, so it must then be closed with `wip`.
        if decl.obligations_terminator is not None:
            block_wip = any(
                c.data["terminator"] == "wip" or self._steps_use_wip(c.data["steps"])
                for c in decl.obligations
            )
            self.check_terminator(
                decl.obligations_terminator, block_wip, f"include {'::'.join(decl.path)}"
            )


def check_module(module: Module, loader: Any = None) -> list[TypeIssue]:
    return Checker(module, loader=loader).check()
