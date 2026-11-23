TARGET_DIR := $(shell cargo metadata --format-version 1 | python3 -c "import sys, json; print(json.load(sys.stdin)['target_directory'])")
DEBUG_DIR  := $(TARGET_DIR)/debug
CC         ?= cc
CFLAGS     := -Wall -Wextra -std=c11

.PHONY: all rust florb krenth clean

all: florb krenth

rust:
	cargo build

florb: rust
	$(CC) $(CFLAGS) examples/florb.c -Iexamples -L"$(DEBUG_DIR)" -lharper_raw -Wl,-rpath,"$(DEBUG_DIR)" -o "$(DEBUG_DIR)/florb"
	"$(DEBUG_DIR)/florb"

krenth: rust
	$(CC) $(CFLAGS) examples/krenth.c -Iexamples -L"$(DEBUG_DIR)" -lharper_safe -Wl,-rpath,"$(DEBUG_DIR)" -o "$(DEBUG_DIR)/krenth"
	"$(DEBUG_DIR)/krenth"
