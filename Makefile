boot ?= uefi
BUILD_ARGS = -Z build-std=core,alloc,compiler_builtins --target x86_64.json
arch = x86_64
FS_IMG = $(CURDIR)/user-rs/target/$(arch)/release/fs.img
mode ?= release
feature ?= monolithic
# feature ?= hybrid

build: ncore bootloader misc/libmymalloc.a fs-img 

ncore:
	cd user-components && cargo build
	cd Ncore && cargo build $(BUILD_ARGS) --no-default-features --features $(feature)

bootloader:
	cd boot && cargo build

fs-img:
	cd user-rs && make build feature=$(feature)
	cd musl && make all
	cd user-c && make all
	rm -f $(FS_IMG)
	cd rcore-fs-use && cargo run --release -- -s $(CURDIR)/user-rs/src/bin -t $(CURDIR)/user-rs/target/$(arch)/$(mode)/

misc/libmymalloc.a:
	cd mymalloc && cargo build
	cp mymalloc/target/x86_64-unknown-none/debug/libmymalloc.a misc/

test: build
	cd kernel && cargo test -- --${boot}

run: build
	cd boot && cargo run -- --${boot}

re:
	cd musl && make re
	cd user-c && make re
	cd rcore-fs-use && cargo clean && cargo build
	make run

gdb: build
	cd boot && cargo run -- --gdb --${boot}

doc:
	cd kernel && cargo doc --document-private-items --open

clean:
	cd Ncore && cargo clean
	cd boot && cargo clean
	cd user-components && cargo clean
	cd hybrid-objects && cargo clean
	cd hybrid-syscalls && cargo clean
	cd monolithic-objects && cargo clean
	cd monolithic-syscalls && cargo clean
	cd mymalloc && cargo clean
	cd user-c && make clean
	cd user-rs && make clean
	rm misc/libmymalloc.a

count:
	cloc . --exclude-dir=crates,target,musl,tutorial,sqlite3,busybox
	# cloc . --exclude-dir=crates,target,musl,user-c,user-rs, tutorial

# test