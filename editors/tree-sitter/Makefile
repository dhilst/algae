# Makefile for the Algae v2 tree-sitter grammar.
#
# Requires the tree-sitter CLI (pinned to 0.26.9 in tree-sitter.json /
# package.json). Install with: `npm install` (uses the pinned devDependency)
# or a matching system package.

TS ?= tree-sitter

# Files produced by `tree-sitter generate`.
GEN := src/parser.c src/grammar.json src/node-types.json

.PHONY: all generate test parse parse-stdlib clean

all: generate

# Regenerate src/parser.c (+ grammar.json, node-types.json, headers).
generate: grammar.js
	$(TS) generate

# Run the corpus tests under test/corpus/.
test: generate
	$(TS) test

# Parse every extracted stdlib module and fail on any ERROR/MISSING node.
parse-stdlib: generate
	@status=0; \
	for f in test/stdlib/*.alg test/*.alg; do \
		[ -f "$$f" ] || continue; \
		if $(TS) parse -q "$$f" >/dev/null 2>&1; then \
			echo "ok    $$f"; \
		else \
			echo "ERROR $$f"; status=1; \
		fi; \
	done; \
	exit $$status

# Parse a single file: make parse FILE=path/to/file.alg
parse: generate
	$(TS) parse $(FILE)

clean:
	rm -rf src/parser.c src/grammar.json src/node-types.json src/tree_sitter
