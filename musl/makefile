MUSL_VERSION ?= 1.2.5
MUSL_URL ?= https://musl.libc.org/releases/musl-$(MUSL_VERSION).tar.gz
MUSL_PREFIX ?= $(CURDIR)/musl
MUSL_TAR := musl-$(MUSL_VERSION).tgz
MUSL_TAR_PATH := src/$(MUSL_TAR)
MUSL_DIR = src/$(arch)/musl-$(MUSL_VERSION)/

arch := x86_64

# 下载musl
$(MUSL_TAR_PATH):
	@mkdir -p src
	wget -O $(MUSL_TAR_PATH) "$(MUSL_URL)"

# 解压musl
$(MUSL_DIR): $(MUSL_TAR_PATH)
	@mkdir -p src/$(arch)
	cd src/$(arch) && tar xvf ../../$(MUSL_TAR_PATH)


# 编译musl
musl: build/$(arch)/bin/musl-gcc

build/$(arch)/bin/musl-gcc:
	cd $(MUSL_DIR) && \
	CFLAGS='-Os -ffunction-sections -fdata-sections' \
	LDFLAGS='-Wl,--gc-sections' \
	./configure --prefix=$(CURDIR)/build/$(arch) &&\
	make -j$(shell nproc) && \
	make install && \
	make distclean &&\
	mkdir ../../../user-c/bin 

# 下载并编译musl
all: $(MUSL_TAR_PATH) $(MUSL_DIR) musl

clean: 
	rm -rf build/$(arch)
	rm -rf src/$(arch)