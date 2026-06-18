"""AST nodes for the equational .alg language."""

from __future__ import annotations

from dataclasses import dataclass, field, fields, is_dataclass
from typing import Any


@dataclass(slots=True)
class Module:
    declarations: list[Any]
    trailing_comments: list[str] = field(default_factory=list)


@dataclass(slots=True)
class SortDecl:
    name: str
    kind_expr: Any  # kind expression over the Sort primitive: Sort, Sort → Sort, …
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class ParamDecl:
    name: str
    kind_expr: Any  # abstract sort/constructor kind: Sort, Sort → Sort, …
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class OpDecl:
    name: str
    domain: list[Any]
    codomain: Any
    partial: bool = False
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class EqDecl:
    expr: Any  # an equation lhs = rhs (a `binary` node with op "=")
    name: str
    params: list[Any] = field(default_factory=list)  # schematic binder params
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class PropDecl:
    expr: Any  # an equation lhs = rhs; an instantiation obligation, not trusted
    name: str
    params: list[Any] = field(default_factory=list)  # schematic binder params
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class LemmaDecl:
    expr: Any  # an equation lhs = rhs
    name: str
    proof: Any = None  # node("proof", steps=[...]) — parsed, never discharged
    params: list[Any] = field(default_factory=list)  # schematic binder params
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class LetDecl:
    name: str
    expr: Any
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class IncludeDecl:
    path: list[str]  # module path: include foo::bar  →  ["foo", "bar"]
    bindings: list[Any] = field(default_factory=list)  # (param_name, type_expr) from `with`
    obligations: list[Any] = field(default_factory=list)  # node("case", ...) proving props
    obligations_terminator: str | None = None  # "qed"/"wip" closing the `props` block
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class OpenDecl:
    path: list[str]
    names: list[str]  # the explicit names brought into scope unqualified
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class AliasDecl:
    alias: str
    path: list[str]  # alias bar = foo::bar;  →  alias="bar", path=["foo","bar"]
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class RuleDecl:
    name: str
    params: list[Any]  # list of (param_name, type_or_kind_expr) tuples
    premises: list[Any]  # node("rule_premise", name=str, prop=sequent) blocks
    conclusion: Any  # prop node
    line: int = 0
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class Node:
    kind: str
    data: dict[str, Any]


def node(kind: str, **data: Any) -> Node:
    return Node(kind, data)


def to_jsonable(value: Any) -> Any:
    if is_dataclass(value):
        result = {"kind": value.__class__.__name__}
        for field in fields(value):
            result[field.name] = to_jsonable(getattr(value, field.name))
        return result
    if isinstance(value, list):
        return [to_jsonable(item) for item in value]
    if isinstance(value, tuple):
        return [to_jsonable(item) for item in value]
    if isinstance(value, dict):
        return {key: to_jsonable(item) for key, item in value.items()}
    return value
