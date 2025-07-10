CC = gcc
CFLAGS = -Wall -Wextra -std=c99
TARGET = test_ffi
RUST_LIB_STATIC = target/debug/libbip353.a
RUST_LIB_DYNAMIC = target/debug/libbip353.so
INCLUDES = -I./include
LIBS = -ldl -lpthread -lm

all: $(TARGET)

# Build with dynamic linking (easier to get working first)
$(TARGET): test_ffi.c $(RUST_LIB_DYNAMIC)
	$(CC) $(CFLAGS) $(INCLUDES) -o $(TARGET) test_ffi.c -L./target/debug -lbip353 $(LIBS)

# Alternative: Build with static linking (if you update Cargo.toml)
$(TARGET)-static: test_ffi.c $(RUST_LIB_STATIC)
	$(CC) $(CFLAGS) $(INCLUDES) -o $(TARGET) test_ffi.c $(RUST_LIB_STATIC) $(LIBS)

# Build the Rust library
$(RUST_LIB_DYNAMIC):
	cargo build --features ffi

$(RUST_LIB_STATIC):
	cargo build --features ffi

clean:
	rm -f $(TARGET)
	cargo clean

test: $(TARGET)
	LD_LIBRARY_PATH=./target/debug ./$(TARGET)

# For debugging
debug: CFLAGS += -g -DDEBUG
debug: $(TARGET)

.PHONY: all clean test debug