test_zex:
	cargo test test_zex --release -- --nocapture
test_fuse:
	cargo test test_fuse --features "trace-interconnect" --release -- --nocapture
