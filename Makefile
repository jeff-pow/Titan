EXE=Quintessence

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

openbench:
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)
