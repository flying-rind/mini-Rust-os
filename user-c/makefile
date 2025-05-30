MUSL_DIR = ../musl-1.2.5
arch = x86_64
mode ?= release

# 编译musl
musl: musl/build/$(arch)/$(mode)/bin/musl-gcc

musl/build/$(arch)/debug/bin/musl-gcc:
	cd $(MUSL_DIR) && \
	./configure --prefix=$(CURDIR)/build/$(arch)/$(mode) \
		--enable-debug \
		--enable-optimize=0
	make -j$(shell nproc) &&\
	make install &&\
	make distclean &&\
	cp $(MUSL_DIR)/build/$(arch)/$(mode)/lib/libc.so user-c/bin/$(arch)/$(mode)

musl/build/$(arch)/release/bin/musl-gcc:
	cd $(MUSL_DIR) && \
	CFLAGS='-Os -ffunction-sections -fdata-sections' \
	LDFLAGS='-Wl,--gc-sections' \
	./configure --prefix=$(CURDIR)/build/$(arch)/$(mode) &&\
	make -j$(shell nproc) && \
	make install && \
	make distclean &&\
	cp $(MUSL_DIR)/build/$(arch)/$(mode)/lib/libc.so ../user-c/bin/$(arch)/$(mode)



MUCC = $(MUSL_DIR)/build/$(arch)/$(mode)/bin/musl-gcc
APP_TARGET = bin/$(arch)/$(mode)/

hello:
	$(MUCC) -static src/hello.c -o $(APP_TARGET)/hello
