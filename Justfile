cp:
	cp ../llvm-project/test.o test.o

objdump filename: cp
	cargo run --bin objdump -- {{filename}}

ld: cp
	cargo run --bin linker -- test.o

run: ld
	cargo run --bin loader -- test

# Create code coverage. You should `cargo install rustfilt, cargo-binutils` and `rustup component add llvm-tools`
coverage:
	#!/bin/bash
	export LLVM_PROFILE_FILE="target/%p-%m.profraw"
	export RUSTFLAGS="-C instrument-coverage"
	
	cargo build
	cargo test -q
	cargo profdata -- merge -sparse target/*.profraw -o target/coverage.profdata
	cargo cov -- export \
		$( \
			for file in \
				$( \
				cargo test --tests --no-run --message-format=json \
					| jq -r "select(.profile.test == true) | .filenames[]" \
					| grep -v dSYM - \
				); \
			do \
				printf "%s %s " -object $file; \
			done \
		) \
		--instr-profile=target/coverage.profdata --format lcov > target/lcov.info
	rm target/*.profraw