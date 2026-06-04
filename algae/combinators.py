"""Small parser-combinator helpers used by the .alg parser."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable, Generic, TypeVar

T = TypeVar("T")
U = TypeVar("U")


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

    def map(self, func: Callable[[T], U]) -> "Parser[U]":
        def parse(state: State) -> tuple[U, State]:
            value, next_state = self(state)
            return func(value), next_state

        return Parser(parse, self._label)

    def bind(self, func: Callable[[T], "Parser[U]"]) -> "Parser[U]":
        def parse(state: State) -> tuple[U, State]:
            value, next_state = self(state)
            return func(value)(next_state)

        return Parser(parse)

    def label(self, expected: str) -> "Parser[T]":
        return Parser(self.func, expected)

    def optional(self) -> "Parser[T | None]":
        def parse(state: State) -> tuple[T | None, State]:
            try:
                return self(state)
            except ParseFailure as exc:
                if exc.state.index != state.index:
                    raise
                return None, state

        return Parser(parse)

    def many(self) -> "Parser[list[T]]":
        def parse(state: State) -> tuple[list[T], State]:
            values: list[T] = []
            current = state
            while True:
                try:
                    value, current = self(current)
                except ParseFailure as exc:
                    if exc.state.index != current.index:
                        raise
                    return values, current
                values.append(value)

        return Parser(parse)

    def __or__(self, other: "Parser[T]") -> "Parser[T]":
        def parse(state: State) -> tuple[T, State]:
            try:
                return self(state)
            except ParseFailure as left:
                try:
                    return other(state)
                except ParseFailure as right:
                    if right.state.index > left.state.index:
                        raise right
                    if left.state.index > right.state.index:
                        raise left
                    raise ParseFailure(left.state, f"{left.expected} or {right.expected}") from right

        return Parser(parse)


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


def sequence(*parsers: Parser[object]) -> Parser[list[object]]:
    def parse(state: State) -> tuple[list[object], State]:
        values: list[object] = []
        current = state
        for parser in parsers:
            value, current = parser(current)
            values.append(value)
        return values, current

    return Parser(parse)
