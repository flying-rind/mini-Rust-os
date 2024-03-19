boot ?= uefi
BUILDARGS = -Z build-std=core,alloc,compiler_builtins
build:
	cd user && make build 
	cd kernel && cargo build $(BUILDARGS)
	cd boot && cargo build

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

re:
	cd user && make clean
	cd kernel && cargo clean
	make run