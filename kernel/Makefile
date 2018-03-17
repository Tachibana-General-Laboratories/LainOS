arch := aarch64-elf
TARGET ?= raspi3-lainos
CROSS := $(arch)

CC := $(CROSS)-gcc
CCFLAGS ?= -Wall -O2 -ffreestanding -nostdinc -nostdlib -nostartfiles -pie -fpie

LDFLAGS ?= --gc-sections -static -nostdlib -nostartfiles --no-dynamic-linker
XARGO ?= CARGO_INCREMENTAL=0 RUST_TARGET_PATH="$(shell pwd)" xargo

LD_LAYOUT := ext/layout.ld

BUILD_DIR_DEBUG := target/build_debug
BUILD_DIR_RELEASE := target/build_release

ifeq ($(DEBUG),1)
BUILD_DIR := $(BUILD_DIR_DEBUG)
else
BUILD_DIR := $(BUILD_DIR_RELEASE)
endif

RUST_BINARY := $(shell cat Cargo.toml | grep name | cut -d\" -f 2 | tr - _)
RUST_BUILD_DIR := target/$(TARGET)
RUST_DEBUG_LIB := $(RUST_BUILD_DIR)/debug/lib$(RUST_BINARY).a
RUST_RELEASE_LIB := $(RUST_BUILD_DIR)/release/lib$(RUST_BINARY).a
RUST_LIB := $(BUILD_DIR)/$(RUST_BINARY).a

RUST_DEPS = Xargo.toml Cargo.toml build.rs $(LD_LAYOUT) src/*
EXT_DEPS := $(BUILD_DIR)/start.o

KERNEL := $(BUILD_DIR)/$(RUST_BINARY)

.PHONY: all qemu clean check test

VPATH = ext

all: $(KERNEL).img
	cp -f $(KERNEL).img ../kernel8.img

qemu: all
	qemu-system-aarch64 -M raspi3 \
		-m 1024 \
		-display sdl,gl=on -sdl \
		-drive file=fs.img,if=sd,format=raw \
		-serial stdio \
		-kernel $(KERNEL).img

qemu2: all
	sudo qemu-system-aarch64 -M raspi3 \
		-m 1024 \
		-smp 4 \
		-accel tcg,thread=multi \
		-display sdl,gl=on -sdl \
		-drive file=../fs.img,if=sd,format=raw \
		-serial /dev/tty5 \
		-serial stdio \
		-kernel $(KERNEL).img

test:
	cargo test

clean:
	rm -rf $(BUILD_DIR)
	$(XARGO) clean

check:
	$(XARGO) check --target=$(TARGET)

$(BUILD_DIR):
	mkdir -p $@

$(BUILD_DIR)/%.o: %.S | $(BUILD_DIR)
	$(CROSS)-gcc $(CCFLAGS) -c $< -o $@

$(RUST_DEBUG_LIB): $(RUST_DEPS)
	$(XARGO) build --target=$(TARGET)
$(RUST_RELEASE_LIB): $(RUST_DEPS)
	$(XARGO) build --verbose --release --target=$(TARGET)

ifeq ($(DEBUG),1)
$(RUST_LIB): $(RUST_DEBUG_LIB) | $(BUILD_DIR)
	cp $< $@
else
$(RUST_LIB): $(RUST_RELEASE_LIB) | $(BUILD_DIR)
	cp $< $@
endif

$(KERNEL).elf: $(EXT_DEPS) $(RUST_LIB)
	$(CROSS)-ld $(LDFLAGS) $^ -T $(LD_LAYOUT) -o $@
	$(CROSS)-strip $@
$(KERNEL).img: $(KERNEL).elf
	$(CROSS)-objcopy -O binary $< $@

