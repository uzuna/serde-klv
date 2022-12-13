.PHONY: fmt
fmt:
	cargo fmt
	cargo clippy --fix --allow-staged

.PHONY: check-fmt
check-fmt:
	cargo fmt --check
	cargo clippy

.PHONY: bench
bench:
	cargo bench --feature uasdls

.PHONY: test
test:
	cargo test --all -- --nocapture
