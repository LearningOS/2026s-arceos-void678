TARGET := mapfile

CC := $(shell command -v riscv64-linux-musl-gcc >/dev/null 2>&1 && echo riscv64-linux-musl-gcc || echo 'zig cc -target riscv64-linux-musl')
STRIP := $(shell command -v riscv64-linux-musl-strip >/dev/null 2>&1 && echo riscv64-linux-musl-strip || echo true)

all: $(TARGET)

%: %.c
	$(CC) -static $< -o $@
	$(STRIP) $@

clean:
	@rm -rf ./$(TARGET)
