MUSL_VERSION ?= 1.2.5
MUSL_URL ?= https://musl.libc.org/releases/musl-$(MUSL_VERSION).tar.gz
MUSL_TAR := musl-$(MUSL_VERSION).tar.gz
MUSL_DIR = musl-$(MUSL_VERSION)

# Download musl src code.
$(MUSL_TAR):
	wget "$(MUSL_URL)"

# Download and unzip the musl src code.
$(MUSL_DIR): $(MUSL_TAR)
	tar xvf $(MUSL_TAR)
	rm $(MUSL_TAR)

#Download and compile musl, get ready for busybox.
musl: $(MUSL_DIR) $(MUSL_DIR)/bin/musl-gcc prepare4busybox

$(MUSL_DIR)/bin/musl-gcc:
	cp -r ../mymalloc $(MUSL_DIR)/src/malloc/
	cd $(MUSL_DIR) && \
	CFLAGS='-Os -ffunction-sections -fdata-sections' \
	LDFLAGS='-Wl,--gc-sections' \
	./configure --prefix=$(CURDIR)/$(MUSL_DIR) --with-malloc=mymalloc &&\
	make -j$(shell nproc) && \
	make install

# Required by busybox.
prepare4busybox:
	cd $(MUSL_DIR)/bin &&\
	ln -s `which ar` musl-ar &&\
	ln -s `which strip` musl-strip

	cd $(MUSL_DIR)/include &&\
	ln -s /usr/include/linux linux &&\
	ln -s /opt/package/usr/include/asm asm &&\
	ln -s /usr/include/asm-generic asm-generic

all: musl

clean: 
	rm -rf $(MUSL_DIR)