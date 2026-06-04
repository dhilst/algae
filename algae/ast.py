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
    names: list[str]
    values: list[str] | None = None
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class OpDecl:
    name: str
    domain: list[Any]
    codomain: Any
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class VarDecl:
    name: str
    sort: Any
    leading_comments: list[str] = field(default_factory=list)
    trailing_comment: str | None = None


@dataclass(slots=True)
class AxiomDecl:
    expr: Any
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
