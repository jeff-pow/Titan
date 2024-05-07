EXE = Titan
LXE = titan

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(LXE)
endif

openbench:
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)

avx512:
	cargo rustc --release --features avx512 -- -C target-cpu=native --emit link=$(NAME)

bench:
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)
	./$(NAME) bench

ancient:
	cargo rustc --release -- -C target-cpu=x86-64 --emit link=$(NAME)
