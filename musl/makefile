MUSL_VERSION ?= 1.2.0
MUSL_URL ?= https://musl.libc.org/releases/musl-$(MUSL_VERSION).tar.gz
MUSL_TAR := musl-$(MUSL_VERSION).tar.gz
MUSL_DIR = musl-$(MUSL_VERSION)
MUCC_DIR = ./musl-$(MUSL_VERSION)/bin
MUINC_DIR = ./musl-$(MUSL_VERSION)/include

# Download musl src code.
$(MUSL_TAR):
	wget "$(MUSL_URL)"

# Download and unzip the musl src code.
$(MUSL_DIR): $(MUSL_TAR)
	tar xvf $(MUSL_TAR)

#Download and compile musl, get ready for busybox.
musl: $(MUSL_DIR)/bin/musl-gcc #prepare4busybox

$(MUSL_DIR)/bin/musl-gcc: $(MUSL_DIR)
	# cp -r mymalloc $(MUSL_DIR)/src/malloc/
	# cp __libc_start_main.c $(MUSL_DIR)/src/env/
	# cp new-makefile $(MUSL_DIR)/Makefile
	cd $(MUSL_DIR) && \
	./configure --prefix=$(CURDIR)/$(MUSL_DIR) \
	# --with-malloc=mymalloc &&\
	make -j$(shell nproc) && \
	make install

# Required by busybox.
prepare4busybox: $(MUCC_DIR)/musl-ar $(MUCC_DIR)/musl-strip $(MUINC_DIR)/linux $(MUINC_DIR)/asm $(MUINC_DIR)/asm-generic
	
$(MUCC_DIR)/musl-ar:
	cd $(MUSL_DIR)/bin &&\
	ln -s `which ar` musl-ar

$(MUCC_DIR)/musl-strip:
	cd $(MUSL_DIR)/bin &&\
	ln -s `which strip` musl-strip

$(MUINC_DIR)/linux:
	cd $(MUSL_DIR)/include &&\
	ln -s /usr/include/linux linux

$(MUINC_DIR)/asm:
	cd $(MUSL_DIR)/include &&\
	ln -s /opt/package/usr/include/asm asm

$(MUINC_DIR)/asm-generic:
	cd $(MUSL_DIR)/include &&\
	ln -s /usr/include/asm-generic asm-generic

all: musl

re:
	make clean
	make all

clean: 
	rm -rf $(MUSL_DIR)