boot ?= uefi
BUILD_ARGS = -Z build-std=core,alloc,compiler_builtins --target x86_64.json
arch = x86_64
FS_IMG = $(CURDIR)/user-rs/target/$(arch)/release/fs.img
mode ?= release

build: ncore bootloader fs-img

ncore:
	cd user-components && cargo build
	cd Ncore && cargo build $(BUILD_ARGS)

bootloader:
	cd boot && cargo build

fs-img:
	cd user-rs && make build
	rm -f $(FS_IMG)
	cd rcore-fs-use && cargo run --release -- -s $(CURDIR)/user-rs/src/bin -t $(CURDIR)/user-rs/target/$(arch)/release/

test: build
	cd kernel && cargo test -- --${boot}

run: build
	cd boot && cargo run -- --${boot}

gdb: build
	cd boot && cargo run -- --gdb --${boot}

doc:
	cd kernel && cargo doc --document-private-items --open

clean:
	cd user-rs && make clean
	cd Ncore && cargo clean
	cd boot && cargo clean
	cd user-components && cargo clean
	cd crates/trapframe-rs && cargo clean

count:
	# cloc . --exclude-dir=target,book,build,crates,musl,source
	cloc . --exclude-dir=crates,target
