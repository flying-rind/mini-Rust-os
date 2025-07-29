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

apps: hello raw blank

# %: src/%.c
# 	$(MUCC) -static -I$(MUINC) -o bin/$@ $<

hello: src/hello.c
	$(MUCC) -static -I$(MUINC) -o bin/hello src/hello.c

raw: src/raw.c
	gcc -nostdlib -static src/raw.c -o bin/raw

blank: src/blank.c
	$(MUCC) -static -I$(MUINC) -o bin/blank src/blank.c