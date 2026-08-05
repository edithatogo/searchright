.PHONY: bootstrap check test coverage security contracts docs verify package

bootstrap:
	./scripts/bootstrap.sh

check:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --all-features -- -D warnings

contracts:
	python3 scripts/validate_repository.py

test:
	cargo nextest run --workspace --all-features
	cargo test --workspace --doc --all-features

coverage:
	cargo llvm-cov nextest --workspace --all-features --fail-under-lines 90

security:
	cargo deny check
	cargo audit
	zizmor --min-severity medium .github/workflows

verify:
	./scripts/verify.sh

package:
	cargo package -p searchright-cli --locked
	cargo package -p searchright-mcp --locked
