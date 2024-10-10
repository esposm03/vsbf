cp:
	cp ../llvm-project/test.o test.o

objdump filename: cp
	cargo run --bin objdump -- {{filename}}

ld: cp
	cargo run --bin linker -- test.o

run: ld
	cargo run --bin loader -- test
