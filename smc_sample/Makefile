# Adapted from
# https://en.wikibooks.org/wiki/Super_NES_Programming/Initialization_Tutorial
# at 2021-09-02T05:40-04:00

all: minimal.smc

minimal.obj: minimal.asm Header.inc Snes_Init.asm
	wla-65816 -o "$@" "$<"

minimal.smc: minimal.link minimal.obj
	wlalink "$<" "$@"
