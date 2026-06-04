"""Command line interface for the .alg parser."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Sequence

from .ast import to_jsonable
from .format import format_spec
from .parser import ParseError, parse_file


def error_line(path: Path, error: ParseError) -> str:
    return f"{path}: error at {error.line}, Expected {error.expected} found {error.found}"


def check(paths: list[Path]) -> int:
    failed = False
    for path in paths:
        try:
            parse_file(path)
        except ParseError as exc:
            failed = True
            print(error_line(path, exc))
        else:
            print(f"{path}: ok")
    return 1 if failed else 0


def fmt(paths: list[Path], *, ascii: bool, inplace: bool) -> int:
    failed = False
    outputs: list[tuple[Path, str]] = []
    for path in paths:
        try:
            rendered = format_spec(parse_file(path), ascii=ascii)
        except ParseError as exc:
            failed = True
            print(error_line(path, exc), file=sys.stderr)
        else:
            outputs.append((path, rendered))
    if failed:
        return 1
    if inplace:
        for path, rendered in outputs:
            path.write_text(rendered, encoding="utf-8")
    else:
        for index, (_, rendered) in enumerate(outputs):
            if index:
                print()
            print(rendered, end="")
    return 0


def print_ast(paths: list[Path]) -> int:
    failed = False
    results = []
    for path in paths:
        try:
            ast = parse_file(path)
        except ParseError as exc:
            failed = True
            results.append(
                {
                    "file": str(path),
                    "ok": False,
                    "error": {
                        "line": exc.line,
                        "column": exc.column,
                        "expected": exc.expected,
                        "found": exc.found,
                    },
                }
            )
        else:
            results.append({"file": str(path), "ok": True, "ast": to_jsonable(ast)})
    payload = results[0] if len(results) == 1 else results
    print(json.dumps(payload, ensure_ascii=False, indent=2))
    return 1 if failed else 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="algae.py")
    subparsers = parser.add_subparsers(dest="command", required=True)

    check_parser = subparsers.add_parser("check")
    check_parser.add_argument("files", nargs="+", type=Path)

    fmt_parser = subparsers.add_parser("fmt")
    fmt_parser.add_argument("--ascii", action="store_true")
    fmt_parser.add_argument("--inplace", action="store_true")
    fmt_parser.add_argument("files", nargs="+", type=Path)

    print_parser = subparsers.add_parser("print")
    print_parser.add_argument("files", nargs="+", type=Path)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.command == "check":
        return check(args.files)
    if args.command == "fmt":
        return fmt(args.files, ascii=args.ascii, inplace=args.inplace)
    return print_ast(args.files)


if __name__ == "__main__":
    raise SystemExit(main())
