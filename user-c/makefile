MUSL_DIR = ../musl
MUCC_DIR = $(CURDIR)/$(MUSL_DIR)/musl-1.2.5/bin
MUCC = $(MUCC_DIR)/musl-gcc
ELF_TARGET = bin
OBJ_TARGET = obj
# SQLite
SQLITE_URL := https://sqlite.org/2025/sqlite-amalgamation-3500300.zip
SQLITE_TAR := sqlite-amalgamation-3500300.zip
SQLITE_DIR := sqlite3
# Busybox
BUSYBOX_URL = https://github.com/mirror/busybox/archive/refs/heads/master.zip
BUSYBOX_TAR = busybox.zip
BUSYBOX_DIR = busybox

all: $(OBJ_TARGET) $(ELF_TARGET) apps
	
$(OBJ_TARGET) $(ELF_TARGET):
	mkdir -p $@

clean: 
	rm -rf $(SQLITE_DIR) $(SQLITE_TAR)
	rm -rf $(BUSYBOX_DIR) $(BUSYBOX_TAR)
	rm obj/*
	rm bin/*

# Download and extract sqlite3.
sqlite3:  $(SQLITE_TAR)

# Download sqlite3 src code.
$(SQLITE_TAR):
	@echo "Downloading SQLite3..."
	wget $(SQLITE_URL)
	unzip -j $(SQLITE_TAR) -d $(SQLITE_DIR)/  
	@echo "SQLite3 downloaded and extracted."


# Download busybox src code.
$(BUSYBOX_TAR):
	@echo "Downloading busybox"
	wget $(BUSYBOX_URL) -O $(BUSYBOX_TAR)
	unzip $(BUSYBOX_TAR) 
	mv busybox-master busybox
	@echo "Busybox downloaded and extracted."

apps: $(ELF_TARGET)/hello $(ELF_TARGET)/raw $(ELF_TARGET)/blank $(ELF_TARGET)/sqlite-test $(ELF_TARGET)/busybox

$(ELF_TARGET)/%: src/%.c
	$(MUCC) -static -o $@ $<
	strip $@ -o $@

$(ELF_TARGET)/raw: src/raw.c
	gcc -nostdlib -static $< -o $@
	strip $@ -o $@

$(ELF_TARGET)/sqlite-test: src/sqlite-test.c sqlite3
	$(MUCC) -I./sqlite3 src/sqlite-test.c -c -o $(OBJ_TARGET)/sqlite-test.o
	$(MUCC) -I./sqlite3 sqlite3/sqlite3.c -c -o $(OBJ_TARGET)/sqlite3.o
	$(MUCC) --static $(OBJ_TARGET)/sqlite-test.o $(OBJ_TARGET)/sqlite3.o -o $@
	strip $@

# Compile busy box with musl-libc, then copy busybox elf to bin.
$(ELF_TARGET)/busybox: $(BUSYBOX_TAR)
	cp ../misc/.config busybox/.config
	export PATH="$(MUCC_DIR):$$PATH" &&\
	cd busybox && make -j 16
	cp busybox/busybox bin/