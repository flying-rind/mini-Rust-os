MUSL_DIR = ../musl
arch = x86_64
mode ?= release
CFLAGS=-I. -Wall -fno-builtin -ffloat-store


MUCC = $(MUSL_DIR)/build/$(arch)/$(mode)/bin/musl-gcc
APP_TARGET = bin
MUINC = $(MUSL_DIR)/build/$(arch)/$(mode)/include
MULIBC = $(MUSL_DIR)/build/$(arch)/$(mode)/lib/libc.a


all: apps

clean: 
	rm bin/*

apps: hello

%: src/%.c
	$(MUCC) -static -I$(MUINC) -o bin/$@ $<
