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

from .ast import (
    AliasDecl,
    AxiomDecl,
    IncludeDecl,
    LemmaDecl,
    LetDecl,
    Module,
    Node,
    OpDecl,
    OpenDecl,
    RuleDecl,
    SortDecl,
    VarDecl,
    node,
    to_jsonable,
)
from .format import Formatter

NUMERIC = {"ℕ": 0, "ℤ": 1, "ℝ": 2}
BOOL = node("type_builtin", name="𝔹")
NAT = node("type_builtin", name="ℕ")
INT = node("type_builtin", name="ℤ")
PROP = node("type_builtin", name="Prop")  # the type of propositions, for rule predicates
STRING = node("type_builtin", name="String")  # internal; string literals only
COMPARISONS = {"=", "≠", "<", "≤", ">", "≥"}
ORDERINGS = {"<", "≤", ">", "≥"}
BOOL_OPS = {"∧", "∨", "⟹", "⟺"}
ARITHMETIC = {"+", "-", "*", "/", "×"}

_render = Formatter().type_expr
_render_expr = Formatter().expr


def same_term(a: Any, b: Any) -> bool:
    """Structural equality of two term ASTs, ignoring incidental parens (which
    are not nodes). Used to verify a written case subgoal against the computed
    one; hypothesis names are compared elsewhere, only `expr`s flow through here."""
    return to_jsonable(a) == to_jsonable(b)


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
    """A proposition is anything boolean (a concrete law) or of type Prop (a
    predicate application like P(x))."""
    return t is not None and (compatible(t, BOOL) or same_type(t, PROP))


# Substitution and β-reduction over expression Nodes, used to instantiate a
# rule's premises into concrete subgoals at the point of application. Pure and
# structural: they build fresh Nodes and never mutate their inputs.


def subst_expr(expr: Any, subst: dict[str, Node]) -> Any:
    if not isinstance(expr, Node):
        return expr
    kind, data = expr.kind, expr.data
    if kind == "identifier":
        return subst.get(data["name"], expr)
    if kind in ("number", "string", "bool", "bool_symbol", "builtin_set", "unit"):
        return expr
    if kind == "tuple":
        return node("tuple", items=[subst_expr(i, subst) for i in data["items"]])
    if kind == "prime":
        return node("prime", value=subst_expr(data["value"], subst))
    if kind == "call":
        return node(
            "call",
            function=subst_expr(data["function"], subst),
            args=[subst_expr(a, subst) for a in data["args"]],
        )
    if kind == "unary":
        return node("unary", op=data["op"], value=subst_expr(data["value"], subst))
    if kind == "binary":
        return node(
            "binary",
            op=data["op"],
            left=subst_expr(data["left"], subst),
            right=subst_expr(data["right"], subst),
        )
    if kind == "if":
        return node(
            "if",
            condition=subst_expr(data["condition"], subst),
            then=subst_expr(data["then"], subst),
            otherwise=subst_expr(data["otherwise"], subst),
        )
    if kind == "let":
        inner = {k: v for k, v in subst.items() if k != data["name"]}
        return node(
            "let",
            name=data["name"],
            value=subst_expr(data["value"], subst),
            body=subst_expr(data["body"], inner),
        )
    if kind == "let_tuple":
        inner = {k: v for k, v in subst.items() if k not in data["binders"]}
        return node(
            "let_tuple",
            binders=data["binders"],
            value=subst_expr(data["value"], subst),
            body=subst_expr(data["body"], inner),
        )
    if kind in ("lambda", "forall", "exists"):
        bound = {name for name, _ in data["binders"]}
        inner = {k: v for k, v in subst.items() if k not in bound}
        return node(kind, binders=data["binders"], body=subst_expr(data["body"], inner))
    return expr


def beta_reduce(expr: Any) -> Any:
    if not isinstance(expr, Node):
        return expr
    kind, data = expr.kind, expr.data
    if kind == "call":
        function = beta_reduce(data["function"])
        args = [beta_reduce(a) for a in data["args"]]
        binders = function.data["binders"] if function.kind == "lambda" else []
        if function.kind == "lambda" and len(args) == len(binders):
            mapping = {name: arg for (name, _), arg in zip(binders, args)}
            return beta_reduce(subst_expr(function.data["body"], mapping))
        return node("call", function=function, args=args)
    if kind == "tuple":
        return node("tuple", items=[beta_reduce(i) for i in data["items"]])
    if kind == "prime":
        return node("prime", value=beta_reduce(data["value"]))
    if kind == "unary":
        return node("unary", op=data["op"], value=beta_reduce(data["value"]))
    if kind == "binary":
        return node("binary", op=data["op"], left=beta_reduce(data["left"]), right=beta_reduce(data["right"]))
    if kind == "if":
        return node(
            "if",
            condition=beta_reduce(data["condition"]),
            then=beta_reduce(data["then"]),
            otherwise=beta_reduce(data["otherwise"]),
        )
    if kind == "let":
        return node("let", name=data["name"], value=beta_reduce(data["value"]), body=beta_reduce(data["body"]))
    if kind == "let_tuple":
        return node(
            "let_tuple",
            binders=data["binders"],
            value=beta_reduce(data["value"]),
            body=beta_reduce(data["body"]),
        )
    if kind in ("lambda", "forall", "exists"):
        return node(kind, binders=data["binders"], body=beta_reduce(data["body"]))
    return expr


def instantiate_prop(prop: Any, subst: dict[str, Node]) -> Any:
    """Substitute a rule's parameters into a premise/conclusion and β-reduce the
    predicate applications, yielding a concrete subgoal sequent."""

    def go(p: Any) -> Any:
        if isinstance(p, Node) and p.kind == "sequent":
            assumptions = [
                node("assumption", name=a.data["name"], expr=beta_reduce(subst_expr(a.data["expr"], subst)))
                for a in p.data["assumptions"]
            ]
            return node("sequent", assumptions=assumptions, goal=beta_reduce(subst_expr(p.data["goal"], subst)))
        return beta_reduce(subst_expr(p, subst))

    return go(prop)


def is_numeric(t: Node) -> bool:
    return t.kind == "type_builtin" and t.data["name"] in NUMERIC


def widest(a: Node, b: Node) -> Node:
    return a if NUMERIC[a.data["name"]] >= NUMERIC[b.data["name"]] else b


class Checker:
    def __init__(self, module: Module, loader: Any = None) -> None:
        self.module = module
        self.loader = loader
        self.issues: list[TypeIssue] = []
        self.line = 0
        self.sorts: set[str] = set()
        self.sort_arity: dict[str, int] = {}  # parametric sort name → parameter count
        self.type_vars: set[str] = set()  # union of all parametric-sort parameters
        self.enum_values: dict[str, Node] = {}
        self.ops: dict[str, list[tuple[list[Node], Node]]] = {}
        self.vars: dict[str, Node] = {}
        self.lets: dict[str, Node] = {}
        self.rules: dict[str, RuleDecl] = {}
        # Axiom, lemma, and rule names share one namespace.
        self.proof_names: set[str] = set()
        self.aliases: dict[str, list[str]] = {}  # alias name → full module path
        self.included: set[str] = set()  # joined paths of included modules

    def expand_alias(self, module: list[str]) -> list[str]:
        # `alias bar = foo::bar;` lets `bar::x` stand for `foo::bar::x`.
        if module and module[0] in self.aliases:
            return self.aliases[module[0]] + module[1:]
        return list(module)

    def issue(self, message: str) -> None:
        self.issues.append(TypeIssue(self.line, message))

    def check(self) -> list[TypeIssue]:
        for decl in self.module.declarations:
            if isinstance(decl, SortDecl):
                self.line = decl.line
                self.collect_sort(decl)
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
            elif isinstance(decl, VarDecl):
                self.collect_var(decl)
            elif isinstance(decl, RuleDecl):
                # Collect rules up front so `apply` can resolve forward references.
                self.rules.setdefault(decl.name, decl)
        # Third pass: check let/axiom bodies, rule premises and conclusions, and
        # lemma propositions and their proofs. Proof rewrite steps are parsed
        # only; correctness is never verified.
        for decl in self.module.declarations:
            self.line = decl.line
            if isinstance(decl, LetDecl):
                bound = self.synth(decl.expr, {})
                if bound is not None:
                    self.lets[decl.name] = bound
            elif isinstance(decl, AxiomDecl):
                self.register_proof_name(decl.name, "axiom")
                self.check_prop(decl.expr, self.binder_env(decl.params))
            elif isinstance(decl, RuleDecl):
                self.register_proof_name(decl.name, "rule")
                self.check_rule(decl)
            elif isinstance(decl, LemmaDecl):
                self.register_proof_name(decl.name, "lemma")
                env = self.binder_env(decl.params)
                self.check_prop(decl.expr, env)
                if decl.proof is not None:
                    self.check_proof(decl.proof, env)
        return self.issues

    def binder_env(self, params: list[Any]) -> dict[str, Node]:
        # Build a local scope from explicit (name, type) binders, validating each
        # type. Duplicate binder names are reported.
        env: dict[str, Node] = {}
        for name, btype in params:
            self.check_type(btype)
            if name in env:
                self.issue(f"duplicate binder {name}")
            env[name] = btype
        return env

    def register_proof_name(self, name: str | None, kind: str) -> None:
        # Axioms, lemmas, and rules share one namespace; the message names the
        # kind of the colliding declaration.
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
        own_sorts = {name for d in sort_decls for name in d.names}
        own_type_vars = {param for d in sort_decls for param in d.params}
        # `with (T := type)` bindings, resolved in the importing module's scope.
        mapping: dict[str, Node] = {}
        for pname, ptype in decl.bindings:
            self.check_type(ptype)
            mapping[pname] = ptype
        # Unbound parameters stay abstract: register them as opaque type vars.
        self.type_vars.update(own_type_vars - set(mapping))

        def imp(t: Node) -> Node:
            return self.import_type(t, decl.path, own_sorts, mapping)

        for d in sort_decls:
            for name in d.names:
                key = f"{prefix}::{name}"
                self.sorts.add(key)
                self.sort_arity[key] = len(d.params)
            if d.values is not None:
                sort_type = node("type_name", module=list(decl.path), name=d.names[0], args=[])
                for value in d.values:
                    self.enum_values[f"{prefix}::{value}"] = sort_type
        for d in module.declarations:
            if isinstance(d, OpDecl):
                domain = [imp(t) for t in d.domain]
                self.ops.setdefault(f"{prefix}::{d.name}", []).append((domain, imp(d.codomain)))
        self.included.add(prefix)

    def register_open(self, decl: OpenDecl) -> None:
        prefix = "::".join(decl.path)
        if prefix not in self.included:
            self.issue(f"open of un-included module {prefix}")
            return
        for name in decl.names:
            key = f"{prefix}::{name}"
            if key in self.ops:
                if name in self.ops or name in self.enum_values or name in self.sorts:
                    self.issue(f"open name {name} collides with an existing name")
                    continue
                self.ops.setdefault(name, []).extend(self.ops[key])
            elif key in self.sorts:
                self.sorts.add(name)
                self.sort_arity[name] = self.sort_arity.get(key, 0)
            elif key in self.enum_values:
                self.enum_values[name] = self.enum_values[key]
            else:
                self.issue(f"name {name} is not exported by module {prefix}")

    def import_type(
        self, t: Node, namespace: list[str], own_sorts: set[str], mapping: dict[str, Node]
    ) -> Node:
        # Rewrite a type from an included module into the importer's view:
        # substitute `with` bindings for type variables and qualify references
        # to the module's own sorts.
        if not isinstance(t, Node):
            return t
        kind, data = t.kind, t.data
        if kind == "type_name":
            module = data.get("module", [])
            args = [self.import_type(a, namespace, own_sorts, mapping) for a in data.get("args", [])]
            if module:
                return node("type_name", module=module, name=data["name"], args=args)
            name = data["name"]
            if name in mapping and not args:
                return mapping[name]
            if name in own_sorts:
                return node("type_name", module=list(namespace), name=name, args=args)
            return node("type_name", module=[], name=name, args=args)
        if kind in ("type_builtin", "type_unit"):
            return t
        if kind == "type_sequence":
            return node("type_sequence", item=self.import_type(data["item"], namespace, own_sorts, mapping))
        if kind == "type_function":
            return node(
                "type_function",
                left=self.import_type(data["left"], namespace, own_sorts, mapping),
                right=self.import_type(data["right"], namespace, own_sorts, mapping),
                partial=data.get("partial", False),
            )
        if kind in ("type_product", "type_sum"):
            return node(kind, items=[self.import_type(i, namespace, own_sorts, mapping) for i in data["items"]])
        return t

    # Declarations -----------------------------------------------------------

    def collect_sort(self, decl: SortDecl) -> None:
        for name in decl.names:
            if name in self.sorts:
                self.issue(f"duplicate sort {name}")
            self.sorts.add(name)
            self.sort_arity[name] = len(decl.params)
        # A parametric sort's parameters are module-wide type variables, in
        # scope for every op/axiom signature (e.g. `op cons : T → List[T] → …`).
        self.type_vars.update(decl.params)
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
            if name in self.type_vars:
                if args:
                    self.issue(f"type variable {name} cannot take type arguments")
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
        if name in self.vars:
            return self.vars[name]
        if name in self.enum_values:
            return self.enum_values[name]
        if name in self.lets:
            return self.lets[name]
        if name in self.ops:
            return self.op_reference(name)
        self.issue(f"undeclared identifier {name}")
        return None

    def synth_qualified(self, module: list[str], name: str) -> Node | None:
        key = "::".join(self.expand_alias(module) + [name])
        if key in self.enum_values:
            return self.enum_values[key]
        if key in self.ops:
            return self.op_reference(key)
        self.issue(f"undeclared name {key}")
        return None

    def op_reference(self, key: str) -> Node | None:
        # The type of an op named (unapplied): its signature as a function type.
        signatures = self.ops[key]
        if len(signatures) != 1:
            self.issue(f"ambiguous reference to overloaded op {key}")
            return None
        domain, codomain = signatures[0]
        if not domain:
            self.issue(f"nullary op {key} must be called: {key}()")
            return None
        left = domain[0] if len(domain) == 1 else node("type_product", items=domain)
        return node("type_function", left=left, right=codomain)

    def call_op_key(self, function: Any, env: dict[str, Node]) -> str | None:
        # The ops-table key a call targets, or None if `function` is not a
        # (non-shadowed) op name. Handles bare and qualified names.
        if not isinstance(function, Node):
            return None
        if function.kind == "identifier":
            name = function.data["name"]
            if name in env or name in self.vars or name in self.lets:
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
                if not is_prop_like(operand):
                    self.issue(f"{op} requires proposition (𝔹 or Prop) operands, got {_render(operand)}")
                    return None
            return PROP if (same_type(left_t, PROP) or same_type(right_t, PROP)) else BOOL
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

    # Propositions, rules, and proofs ----------------------------------------

    def check_prop(self, prop: Any, env: dict[str, Node]) -> None:
        # Every assumption and the goal must be a proposition (𝔹 or Prop). No
        # logical content is verified.
        if isinstance(prop, Node) and prop.kind == "sequent":
            for assumption in prop.data["assumptions"]:
                t = self.synth(assumption.data["expr"], env)
                if t is not None and not is_prop_like(t):
                    self.issue(f"assumption must be a proposition, got {_render(t)}")
            goal = self.synth(prop.data["goal"], env)
            if goal is not None and not is_prop_like(goal):
                self.issue(f"goal must be a proposition, got {_render(goal)}")
        else:
            t = self.synth(prop, env)
            if t is not None and not is_prop_like(t):
                self.issue(f"proposition must be a 𝔹 or Prop expression, got {_render(t)}")

    def check_rule(self, decl: RuleDecl) -> None:
        seen: set[str] = set()
        param_env: dict[str, Node] = {}
        for pname, ptype in decl.params:
            if pname in seen:
                self.issue(f"duplicate rule parameter {pname}")
            seen.add(pname)
            self.check_type(ptype)
            param_env[pname] = ptype
        for premise in decl.premises:
            self.check_prop(premise, param_env)
            # Hypotheses are named only at `case`; a rule premise must not name
            # its assumptions.
            for assumption in self.premise_assumptions(premise):
                if assumption.data["name"] is not None:
                    self.issue(
                        f"rule premise assumptions must be unnamed; found "
                        f"{assumption.data['name']} :="
                    )
        self.check_prop(decl.conclusion, param_env)

    def check_proof(self, proof: Any, env: dict[str, Node]) -> None:
        for step in proof.data["steps"]:
            if isinstance(step, Node) and step.kind == "apply":
                self.check_apply(step, env)
            # proof_start / proof_rewrite steps are parsed only; their `by`
            # references and rewrite terms are not resolved or verified.

    def check_apply(self, step: Node, env: dict[str, Node]) -> None:
        name = step.data["rule"]
        cases = step.data["cases"]
        rule = self.rules.get(name)
        if rule is None:
            self.issue(f"unknown rule {name}")
            self.check_cases_bodies(cases, env)
            return
        args = step.data["args"]
        if len(args) != len(rule.params):
            self.issue(f"rule {name} expects {len(rule.params)} argument(s), got {len(args)}")
            self.check_cases_bodies(cases, env)
            return
        arg_types = [self.synth(arg, env) for arg in args]
        for (pname, ptype), actual in zip(rule.params, arg_types):
            if actual is not None and not compatible(actual, ptype):
                self.issue(
                    f"argument for {pname} has type {_render(actual)}, expected {_render(ptype)}"
                )
        if len(cases) != len(rule.premises):
            self.issue(
                f"rule {name} has {len(rule.premises)} premise(s) but {len(cases)} case(s) given"
            )
            self.check_cases_bodies(cases, env)
            return
        subst = {pname: arg for (pname, _), arg in zip(rule.params, args)}
        for index, (case, premise) in enumerate(zip(cases, rule.premises)):
            self.check_case(case, premise, subst, index, env)
        self.check_cases_bodies(cases, env)

    def check_case(
        self, case: Node, premise: Any, subst: dict[str, Node], index: int, env: dict[str, Node]
    ) -> None:
        # The author writes the branch's full sequent; verify it equals the
        # subgoal obtained by instantiating the premise (hypothesis names free).
        written = case.data["sequent"]
        self.check_prop(written, env)
        expected = self.as_sequent(instantiate_prop(premise, subst))
        written_hyps = written.data["assumptions"]
        expected_hyps = expected.data["assumptions"]
        if len(written_hyps) != len(expected_hyps):
            self.issue(
                f"case {index + 1} has {len(written_hyps)} hypothesis(es) but the premise "
                f"yields {len(expected_hyps)}"
            )
            return
        for written_hyp, expected_hyp in zip(written_hyps, expected_hyps):
            if not same_term(written_hyp.data["expr"], expected_hyp.data["expr"]):
                self.issue(
                    f"case {index + 1} hypothesis does not match the premise: expected "
                    f"{_render_expr(expected_hyp.data['expr'])}"
                )
        if not same_term(written.data["goal"], expected.data["goal"]):
            self.issue(
                f"case {index + 1} subgoal does not match the premise: expected "
                f"{_render_expr(expected.data['goal'])}"
            )

    @staticmethod
    def as_sequent(prop: Any) -> Node:
        if isinstance(prop, Node) and prop.kind == "sequent":
            return prop
        return node("sequent", assumptions=[], goal=prop)

    @staticmethod
    def premise_assumptions(premise: Any) -> list[Any]:
        if isinstance(premise, Node) and premise.kind == "sequent":
            return premise.data["assumptions"]
        return []

    def check_cases_bodies(self, cases: list[Any], env: dict[str, Node]) -> None:
        for case in cases:
            self.check_proof(node("proof", steps=case.data["steps"]), env)


def check_module(module: Module, loader: Any = None) -> list[TypeIssue]:
    return Checker(module, loader=loader).check()
