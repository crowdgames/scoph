ifeq ($(OS), Windows_NT) 
	OS_TYPE :=Windows
else 
	OS_TYPE :=$(shell uname | tr -d '\n')
endif

ifeq ($(OS_TYPE),Linux)
	STATIC_LIB_FLAGS :=-lX11 -lm -lpthread -lGL -ldl -lrt
else ifeq($(OS_TYPE),Windows)
	# TODO
endif

all: raylib-rewrites

clean:
	cargo clean
	rm -rf build
	rm examples/include/scoph.h

# NOTE: if you want to compile a raylib test, you should download a static library file for raylib and put it in `examples/lib`
raylib-%: scoph build
	cd examples/$@ && gcc -I../../target/ -I../include/ main.c -L../../target/debug -L../lib -lraylib -lscoph $(STATIC_LIB_FLAGS) -o ../../build/$@.test
	./build/$@.test

.PHONY: scoph scoph-test

scoph: scoph-test
	cbindgen -c scoph-ffi/cbindgen.toml  -o examples/include/scoph.h "scoph-ffi"
	cargo build --manifest-path=scoph-ffi/Cargo.toml

scoph-test:
	RUSTFLAGS="-A warnings" cargo test

build:
	mkdir build
