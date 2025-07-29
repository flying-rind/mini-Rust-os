ARCH := x86_64
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(ARCH)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))

BUILD_ARGS := --target $(ARCH).json -Zbuild-std=core,alloc
ifeq ($(MODE), release)
  BUILD_ARGS += --release
endif

all: build strip

elf:
	cargo build $(BUILD_ARGS)

strip:
	strip $(ELFS)

build: elf

clean:
	@cargo clean

.PHONY: all binary disasm build clean
