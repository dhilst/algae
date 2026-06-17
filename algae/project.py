"""Project configuration and module resolution for .alg includes.

A project is rooted at the nearest ancestor directory containing
`alg-project.json`. That file lists `include_path` (directories searched for
modules, relative to the root) and `vendor` (the vendored-modules directory,
default `vendor`). A module path `foo::bar` resolves to `foo/bar.alg`, searched
across the include paths and then the vendor directory. `std` is reserved for
the future vendored standard library (resolved under `vendor/std`).
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from .ast import Module
from .parser import parse_file

PROJECT_FILE = "alg-project.json"


class ModuleError(Exception):
    """A module could not be resolved or formed a cycle."""


def find_project_root(start: Path) -> Path | None:
    """Walk up from `start` (a file or directory) for the project marker."""
    current = start if start.is_dir() else start.parent
    for directory in [current, *current.parents]:
        if (directory / PROJECT_FILE).is_file():
            return directory
    return None


def load_config(root: Path) -> dict[str, Any]:
    data = json.loads((root / PROJECT_FILE).read_text(encoding="utf-8"))
    include_path = data.get("include_path", ["."])
    vendor = data.get("vendor", "vendor")
    return {"include_path": include_path, "vendor": vendor}


@dataclass
class ModuleLoader:
    root: Path
    config: dict[str, Any]
    _cache: dict[str, Module] = field(default_factory=dict)
    _visiting: set[str] = field(default_factory=set)
    _checking: set[str] = field(default_factory=set)

    def begin_check(self, key: str) -> bool:
        """Mark a module as being checked. Returns False if it already is (a
        cycle), in which case the caller must not recurse."""
        if key in self._checking:
            return False
        self._checking.add(key)
        return True

    def end_check(self, key: str) -> None:
        self._checking.discard(key)

    @classmethod
    def for_file(cls, path: Path) -> "ModuleLoader | None":
        """Build a loader for the project containing `path`, or None if the file
        is not inside a project (it then cannot use includes)."""
        root = find_project_root(path.resolve())
        if root is None:
            return None
        return cls(root, load_config(root))

    def resolve(self, path: list[str]) -> Path:
        relative = Path(*path).with_suffix(".alg")
        search = list(self.config["include_path"])
        search.append(self.config["vendor"])  # vendored modules, incl. future std
        for entry in search:
            candidate = self.root / entry / relative
            if candidate.is_file():
                return candidate
        raise ModuleError(f"module {'::'.join(path)} not found")

    def load(self, path: list[str]) -> Module:
        key = "::".join(path)
        if key in self._cache:
            return self._cache[key]
        if key in self._visiting:
            raise ModuleError(f"circular include {key}")
        file_path = self.resolve(path)
        self._visiting.add(key)
        try:
            module = parse_file(file_path)
        finally:
            self._visiting.discard(key)
        self._cache[key] = module
        return module
