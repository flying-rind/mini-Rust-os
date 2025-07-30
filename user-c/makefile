MUSL_DIR = ../musl
arch = x86_64
mode ?= release
CFLAGS=-I. -Wall -fno-builtin -ffloat-store

MUCC = $(MUSL_DIR)/build/$(arch)/$(mode)/bin/musl-gcc
ELF_TARGET = bin
APP_SRC = src
OBJ_TARGET = obj
MUINC = $(MUSL_DIR)/build/$(arch)/$(mode)/include
# SQLite 版本和下载 URL
SQLITE_URL := https://sqlite.org/2025/sqlite-amalgamation-3500300.zip
SQLITE_TAR := sqlite-amalgamation-3500300.zip
SQLITE_DIR := sqlite3

all: apps

clean: 
	rm -rf $(SQLITE_DIR) $(SQLITE_TAR)
	find src -maxdepth 1 -type f ! -name "*.c" -delete
	rm bin/*

# Download sqlite3.
sqlite3: 
	@echo "Downloading SQLite3..."
	wget $(SQLITE_URL)
	unzip -j $(SQLITE_TAR) -d $(SQLITE_DIR)/  
	@echo "SQLite3 downloaded and extracted."

apps: hello raw blank sqlite-test

%: src/%.c
	$(MUCC) -static -I$(MUINC) -o bin/$@ $<
	strip bin/$@ -o bin/$@

raw: src/raw.c
	gcc -nostdlib -static src/raw.c -o bin/raw
	strip bin/raw -o bin/raw

sqlite-test: src/sqlite-test.c sqlite3
	$(MUCC) -I./sqlite3 src/sqlite-test.c -c -o $(OBJ_TARGET)/sqlite-test.o
	$(MUCC) -I./sqlite3 sqlite3/sqlite3.c -c -o $(OBJ_TARGET)/sqlite3.o
	$(MUCC) --static $(OBJ_TARGET)/$@.o $(OBJ_TARGET)/sqlite3.o -o $(ELF_TARGET)/$@




