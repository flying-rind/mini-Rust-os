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
# malloclib
MALLOC_LIB = ../misc/libmymalloc.a
MULIBC = ../musl/musl-1.2.5/lib/libc.a

all: $(OBJ_TARGET) $(ELF_TARGET) apps
	cd bin && find . -type f -name "*.o" -delete
	
$(OBJ_TARGET) $(ELF_TARGET):
	mkdir -p $@

clean: 
	rm -f obj/*
	rm -f bin/*

re:
	make clean
	make all

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

apps: 	$(ELF_TARGET)/sqlite-test 
#$(ELF_TARGET)/hello $(ELF_TARGET)/malloc-test 
#$(ELF_TARGET)/raw $(ELF_TARGET)/blank  $(ELF_TARGET)/mmap-test1 $(ELF_TARGET)/mmap-test2 \
		$(ELF_TARGET)/writev-test $(ELF_TARGET)/getcwd-test $(ELF_TARGET)/open-test $(ELF_TARGET)/ash  #$(ELF_TARGET)/busybox 
	 	

$(ELF_TARGET)/%: src/%.c
	$(MUCC) -c -o $@.o $< 
	$(MUCC) -o $@ --static  $@.o $(MULIBC) $(MALLOC_LIB)

$(ELF_TARGET)/raw: src/raw.c
	gcc -nostdlib -static $< -o $@

$(ELF_TARGET)/sqlite-test: src/sqlite-test.c sqlite3
	CFLAGS="-DSQLITE_DEBUG=1 -DSQLITE_MEMDEBUG=1"
	$(MUCC) -I./sqlite3 src/sqlite-test.c -c -fno-stack-protector -fno-builtin -o  $(OBJ_TARGET)/sqlite-test.o
	$(MUCC) -I./sqlite3 sqlite3/sqlite3.c -c -fno-stack-protector -fno-builtin -o  $(OBJ_TARGET)/sqlite3.o
	$(MUCC) --static -fno-stack-protector -fno-builtin -flto -o $@ $(OBJ_TARGET)/sqlite-test.o  $(OBJ_TARGET)/sqlite3.o $(MULIBC) $(MALLOC_LIB) 

#Compile busy box with musl-libc, then copy busybox elf to bin.
$(ELF_TARGET)/busybox: $(BUSYBOX_TAR)
	cp ../misc/.config busybox/.config
	export PATH="$(MUCC_DIR):$$PATH" &&\
	cd busybox && make -j 16
	cp busybox/busybox bin/

$(ELF_TARGET)/ash:
	cp ../misc/ash bin/

# Now we just use pre-compiled busybox elf.
# $(ELF_TARGET)/busybox:
# 	cp ../misc/busybox bin/
