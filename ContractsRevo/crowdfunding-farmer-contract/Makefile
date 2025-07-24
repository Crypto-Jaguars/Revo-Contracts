.PHONY: build test clean

build:
	cargo build --target wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/crowdfunding_farmer_contract.wasm .

test:
	cargo test -- --nocapture

clean:
	cargo clean
	rm -f crowdfunding_farmer_contract.wasm