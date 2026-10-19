all: scoph test
	./test

clean:
	rm include/scoph.h
	rm lib/libscoph.a
	rm test

test:
	gcc -Iinclude main.c -Llib -lraylib -lscoph -lX11 -lm -lpthread -lGL -ldl -lrt -o $@

.PHONY: scoph test

scoph:
	cargo build --manifest-path=../../scoph-ffi/Cargo.toml
	cp ../../target/debug/libscoph.a lib/libscoph.a
	cbindgen -c ../../scoph-ffi/cbindgen.toml  -o include/scoph.h "../../scoph-ffi"
