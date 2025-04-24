boot ?= uefi
BUILD_ARGS = -Z build-std=core,alloc,compiler_builtins --target x86_64.json
# BUILD_ARGS = -Z build-std=core,alloc,compiler_builtins
arch = x86_64
FS_IMG = ../user/target/$(arch)/release/fs.img
mode ?= release
MUSL_DIR = musl-1.2.5

# 设置musl target
ifeq ($(arch), x86_64)
	MUSL_TARGET := 
else ifeq ($(arch), riscv64)
	MUSL_TARGET := --target=riscv64-linux-gnu
endif

build: Kernel bootloader fs-img

Kernel:
	cd kernel && cargo build $(BUILD_ARGS)

bootloader:
	@cd boot && cargo build

fs-img:
	cd user-rs && make build
	rm -f $(FS_IMG)
	cd easy-fs-fuse && cargo run --release -- -s ../user/src/bin -t ../user/target/$(arch)/release/

test: build
	cd kernel && cargo test -- --${boot}

run: build
	cd boot && cargo run -- --${boot}

gdb: build
	cd boot && cargo run -- --gdb --${boot}

doc:
	cd kernel && cargo doc --document-private-items --open

clean:
	cd user && make clean
	cd kernel && cargo clean
	cd boot && cargo clean
	cd user-components && cargo clean
	cd crates/trapframe-rs && cargo clean

re:
	cd user && make clean
	cd kernel && cargo clean
	make run

# 编译musl
musl: musl/build/$(arch)/$(mode)/bin/musl-gcc

musl/build/$(arch)/debug/bin/musl-gcc:
	cd $(MUSL_DIR) && \
	./configure --prefix=$(CURDIR)/$(MUSL_DIR)/build/$(arch)/$(mode) \
		$(MUSL_TARGET) \
		--with-malloc=kymalloc \
		--enable-debug \
		--enable-optimize=0 &&\
	make -j$(shell nproc) &&\
	make install &&\
	make distclean &&\
	cp $(MUSL_DIR)/build/$(arch)/$(mode)/lib/libc.so user-c/bin/$(arch)/$(mode)

musl/build/$(arch)/release/bin/musl-gcc:
	cd $(MUSL_DIR) && \
	CFLAGS='-Os -ffunction-sections -fdata-sections' \
	LDFLAGS='-Wl,--gc-sections' \
	./configure --prefix=$(CURDIR)/$(MUSL_DIR)/build/$(arch)/$(mode) \
		$(MUSL_TARGET) && \
		--with-malloc=kymalloc \
	make -j$(shell nproc) && \
	make install && \
	make distclean &&\
	cp $(MUSL_DIR)/build/$(arch)/$(mode)/lib/libc.so user-c/bin/$(arch)/$(mode)
