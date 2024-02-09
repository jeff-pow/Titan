EXE=Titan

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

openbench:
	cargo rustc --release --features avx2 -- -C target-cpu=native --emit link=$(NAME)

stable: 
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)

avx2:
	cargo rustc --release --features avx2 -- -C target-cpu=native --emit link=$(NAME)

avx512:
	cargo rustc --release --features avx512 -- -C target-cpu=native --emit link=$(NAME)

bench:
	cargo rustc --release --features avx2 -- -C target-cpu=native --emit link=$(NAME)
	./$(NAME) bench
