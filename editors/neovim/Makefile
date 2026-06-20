# Build the Neovim tree-sitter parser for Algae (.alg) from the committed
# parser.c, so an init.lua only needs to put this directory on the runtimepath.
#
#     make -C editors/neovim        # build parser/alg.so
#     make -C editors/neovim clean

NVIM_DIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))
SRC := $(NVIM_DIR)../tree-sitter/src
PARSER := $(NVIM_DIR)parser/alg.so
CC ?= cc

.PHONY: all clean
all: $(PARSER)

$(PARSER): $(SRC)/parser.c
	mkdir -p $(NVIM_DIR)parser
	$(CC) -o $@ -shared -Os -fPIC -I $(SRC) $(SRC)/parser.c

clean:
	rm -f $(PARSER)
