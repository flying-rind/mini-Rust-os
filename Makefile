boot ?= uefi
BUILD_ARGS = -Z build-std=core,alloc,compiler_builtins --target x86_64.json
ARCH = x86_64
FS_IMG = ../user/target/$(ARCH)/release/fs.img


build: Kernel bootloader fs-img

Kernel:
	cd user && make build 
	cd kernel && cargo build $(BUILD_ARGS)

bootloader:
	@cd boot && cargo build

fs-img:
	@cd user && make build
	@rm -f $(FS_IMG)
	@cd easy-fs-fuse && cargo run --release -- -s ../user/src/bin -t ../user/target/$(ARCH)/release/

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