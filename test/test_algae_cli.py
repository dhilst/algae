from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "algae.py"


class AlgaeCliTests(unittest.TestCase):
    def run_cli(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(CLI), *args],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def test_check_accepts_equational_fixtures(self) -> None:
        result = self.run_cli("check", "test/stack.alg", "test/kvstore.alg", "test/base/container.alg")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("test/stack.alg: ok", result.stdout)
        self.assertIn("test/kvstore.alg: ok", result.stdout)
        self.assertIn("test/base/container.alg: ok", result.stdout)

    def test_check_rejects_old_and_malformed_syntax(self) -> None:
        paths = sorted(str(path.relative_to(ROOT)) for path in (ROOT / "test/reject").glob("*.alg"))
        result = self.run_cli("check", *paths)

        self.assertEqual(result.returncode, 1)
        self.assertIn("Expected", result.stdout)
        self.assertIn("found", result.stdout)
        self.assertNotIn(": ok", result.stdout)

    def test_print_outputs_json_ast(self) -> None:
        result = self.run_cli("print", "test/stack.alg")

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["ast"]["kind"], "Module")
        self.assertEqual(payload["ast"]["declarations"][0]["kind"], "SortDecl")
        self.assertEqual(payload["ast"]["declarations"][2]["kind"], "OpDecl")

    def test_fmt_converts_ascii_aliases_to_unicode(self) -> None:
        source = "\n".join(
            [
                "sort Stack, Elem;",
                "sort Error = {empty_error};",
                "op empty : arrow Stack;",
                "op push : Stack product Elem arrow Stack;",
                "op pop : Stack arrow Stack | Error;",
                "var s : Stack;",
                "var e : Elem;",
                "axiom pop(push(s, e)) neq s;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "alias.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", str(path))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("op empty : → Stack;", result.stdout)
        self.assertIn("Stack × Elem → Stack", result.stdout)
        self.assertIn("pop(push(s, e)) ≠ s", result.stdout)

    def test_fmt_ascii_outputs_keyword_aliases(self) -> None:
        result = self.run_cli("fmt", "--ascii", "test/stack.alg")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Stack product Elem arrow Stack", result.stdout)
        self.assertIn("Stack arrow Elem | Error", result.stdout)
        self.assertNotIn("→", result.stdout)
        self.assertNotIn("×", result.stdout)

    def test_let_expression_parses_and_formats(self) -> None:
        source = "\n".join(
            [
                "sort S;",
                "op f : S -> S;",
                "var x : S;",
                "axiom let y = f(x) in let z = f(y) in f(z) = x;",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "let.alg"
            path.write_text(source, encoding="utf-8")
            check_result = self.run_cli("check", str(path))
            fmt_result = self.run_cli("fmt", str(path))

        self.assertEqual(check_result.returncode, 0, check_result.stderr)
        self.assertEqual(fmt_result.returncode, 0, fmt_result.stderr)
        self.assertIn("axiom let y = f(x) in let z = f(y) in f(z) = x;", fmt_result.stdout)

    def test_fmt_inplace_rewrites_file(self) -> None:
        source = "sort Stack,Elem;op empty:arrow Stack;var s:Stack;axiom empty()=s;\n"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "stack.alg"
            path.write_text(source, encoding="utf-8")
            result = self.run_cli("fmt", "--inplace", str(path))
            rewritten = path.read_text(encoding="utf-8")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertIn("sort Stack, Elem;", rewritten)
        self.assertIn("op empty : → Stack;", rewritten)
        self.assertIn("axiom empty() = s;", rewritten)


if __name__ == "__main__":
    unittest.main()
