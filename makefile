run:
	cargo run --release
run_with_debugger:
	cargo run --release -- --debug

test_zex:
	cargo test test_zex --release -- --nocapture
test_fuse:
	cargo test test_fuse --features "trace-interconnect" --release -- --nocapture
