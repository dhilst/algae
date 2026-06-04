"""Small parser-combinator helpers used by the .alg parser."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable, Generic, TypeVar

T = TypeVar("T")


@dataclass(frozen=True, slots=True)
class Token:
    kind: str
    value: str
    text: str
    line: int
    column: int


@dataclass(frozen=True, slots=True)
class State:
    tokens: tuple[Token, ...]
    index: int = 0

    @property
    def current(self) -> Token:
        return self.tokens[self.index]

    def advance(self, count: int = 1) -> "State":
        return State(self.tokens, self.index + count)


class ParseFailure(Exception):
    def __init__(self, state: State, expected: str) -> None:
        self.state = state
        self.expected = expected
        super().__init__(expected)


class Parser(Generic[T]):
    def __init__(self, func: Callable[[State], tuple[T, State]], label: str | None = None) -> None:
        self.func = func
        self._label = label

    def __call__(self, state: State) -> tuple[T, State]:
        try:
            return self.func(state)
        except ParseFailure as exc:
            if self._label and exc.state.index == state.index:
                raise ParseFailure(exc.state, self._label) from exc
            raise

    def label(self, expected: str) -> "Parser[T]":
        return Parser(self.func, expected)


def satisfy(predicate: Callable[[Token], bool], expected: str) -> Parser[Token]:
    def parse(state: State) -> tuple[Token, State]:
        token = state.current
        if predicate(token):
            return token, state.advance()
        raise ParseFailure(state, expected)

    return Parser(parse, expected)


def token_kind(kind: str, expected: str | None = None) -> Parser[Token]:
    return satisfy(lambda token: token.kind == kind, expected or kind)


def token_value(value: str, expected: str | None = None) -> Parser[Token]:
    return satisfy(lambda token: token.value == value, expected or value)
