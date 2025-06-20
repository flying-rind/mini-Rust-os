MUSL_VERSION ?= 1.2.5
MUSL_URL ?= https://musl.libc.org/releases/musl-$(MUSL_VERSION).tar.gz
MUSL_PREFIX ?= $(CURDIR)/musl
MUSL_TAR := musl-$(MUSL_VERSION).tgz
MUSL_TAR_PATH := src/$(MUSL_TAR)
MUSL_DIR = src/$(arch)/musl-$(MUSL_VERSION)/

arch := x86_64
mode := release

# 下载musl
$(MUSL_TAR_PATH):
	@mkdir -p src
	wget -O $(MUSL_TAR_PATH) "$(MUSL_URL)"

# 解压musl
$(MUSL_DIR): $(MUSL_TAR_PATH)
	@mkdir -p src/$(arch)
	cd src/$(arch) && tar xvf ../../$(MUSL_TAR_PATH)


# 编译musl
musl: build/$(arch)/$(mode)/bin/musl-gcc

build/$(arch)/debug/bin/musl-gcc:
	cd $(MUSL_DIR) && \
	./configure --prefix=$(CURDIR)/build/$(arch)/$(mode) \
		--enable-debug \
		--enable-optimize=0
	make -j$(shell nproc) &&\
	make install &&\
	make distclean &&\
	mkdir ../../../user-c/bin &&\
	cp ../../../build/$(arch)/$(mode)/lib/libc.so ../../../../user-c/bin/

build/$(arch)/release/bin/musl-gcc:
	cd $(MUSL_DIR) && \
	CFLAGS='-Os -ffunction-sections -fdata-sections' \
	LDFLAGS='-Wl,--gc-sections' \
	./configure --prefix=$(CURDIR)/build/$(arch)/$(mode) &&\
	make -j$(shell nproc) && \
	make install && \
	make distclean &&\
	mkdir ../../../user-c/bin &&\
	cp ../../../build/$(arch)/$(mode)/lib/libc.so ../../../../user-c/bin/

# 下载并编译musl
all: $(MUSL_TAR_PATH) $(MUSL_DIR) musl

clean: 
	rm -rf build/$(arch)
	rm -rf src/$(arch)