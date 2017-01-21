run:
	cargo run --release roms/128-0.rom roms/128-1.rom
run_with_debugger:
	RUST_LOG=debug cargo run --release roms/128-0.rom roms/128-1.rom

test_zex:
	cargo test test_zex --release -- --nocapture
test_fuse:
	cargo test test_fuse --features "trace-interconnect" --release -- --nocapture
