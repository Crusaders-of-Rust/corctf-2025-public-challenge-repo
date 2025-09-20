# Makefile to build pid.c into WebAssembly using clang/wasm-ld
# Requirement: Use clang at /opt/homebrew/Cellar/llvm/19.1.0/bin/clang

CLANG := /opt/homebrew/Cellar/llvm/19.1.0/bin/clang
TARGET := pid.wasm
SRC := pid.c

CFLAGS := -O3 -fvisibility=hidden -nostdlib -target wasm32
LDFLAGS := -Wl,--no-entry -Wl,--export=controller_step -Wl,--strip-all -Wl,--gc-sections

.PHONY: all clean

all: $(TARGET)

$(TARGET): $(SRC)
	$(CLANG) $(CFLAGS) $(SRC) -o $(TARGET) $(LDFLAGS)

clean:
	rm -f $(TARGET)
