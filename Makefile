.PHONY: fmt
fmt:
	cargo fmt
	git add -u
	cargo clippy --fix --allow-staged --all-features

.PHONY: check-fmt
check-fmt:
	cargo fmt --check
	cargo clippy

.PHONY: bench
bench:
	cargo bench --feature uasdls

.PHONY: test
test:
	cargo test --all-features -- --nocapture
