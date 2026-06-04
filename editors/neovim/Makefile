CC ?= cc
CFLAGS ?= -O2
SRC := tree-sitter-alg/src

all: parser/alg.so

parser/alg.so: $(SRC)/parser.c
	mkdir -p parser
	$(CC) $(CFLAGS) -fPIC -shared -I$(SRC) -o $@ $<

# Regenerate src/parser.c from grammar.js (requires the tree-sitter CLI).
generate:
	cd tree-sitter-alg && tree-sitter generate

clean:
	rm -rf parser

.PHONY: all generate clean
