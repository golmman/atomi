PROVARI ?= /tmp/atomi-oracle/provari/target/release/provari
PROVARI_STRETCH ?= /tmp/atomi-oracle/provari-v0.2.0/target/release/provari
LEGACY_SOLVER ?= /tmp/atomic_solver-5f85133/target/release/atomic_solver
PERNODE_CASES ?= core,1eb80b92f92c,db6a741c723f
PERNODE_EVALS ?= 1000000,10000000
PERNODE_MEMORY ?= 192
PERNODE_OUT ?= /tmp/atomi-pernode.json

.PHONY: test test-full test-lite lint oracle oracle-stretch veto bench pernode

test:
	python3 scripts/check_licenses.py && cargo test --release

test-full: test
	cargo test --release -- --ignored
	$(MAKE) veto

test-lite:
	cargo test

lint:
	python3 scripts/check_goal.py
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo doc --no-deps

oracle:
	scripts/build_oracle.sh

oracle-stretch:
	scripts/build_oracle.sh --ref v0.2.0

veto:
	cargo build --release && python3 scripts/veto.py --atomi target/release/atomi --provari $(PROVARI)

bench:
	python3 bench/run_bench.py \
	  --provari $(PROVARI) \
	  --provari-stretch $(PROVARI_STRETCH) \
	  --legacy-solver $(LEGACY_SOLVER) \
	  --atomi target/release/atomi

pernode:
	python3 bench/run_pernode.py --atomi target/release/atomi \
	  --cases $(PERNODE_CASES) --evals $(PERNODE_EVALS) \
	  --memory $(PERNODE_MEMORY) --out $(PERNODE_OUT)
