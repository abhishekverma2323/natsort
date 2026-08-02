SHELL := /bin/bash

PYTHON ?= python3
VENV ?= .port-venv
VENV_PYTHON := $(VENV)/bin/python
CARGO_MANIFEST := rust-port/Cargo.toml
SOURCE_COMMIT := b543bdce8771b6e7a7dae0c6745ddf7e80299797

.PHONY: \
	help setup build build-debug fmt test-rust clippy \
	verify-tests test-original verify \
	cli-diff cli-check fuzz fuzz-final-check \
	bench bench-final audit-final \
	docker-build docker-verify verify-evidence submission-check \
	evidence-summary clean

help:
	@echo "Port Mortem 2026 — natsort Python → Rust"
	@echo
	@echo "Core:"
	@echo "  make setup             Create the verification Python environment"
	@echo "  make build             Build all Rust release binaries"
	@echo "  make verify            Hash-check tests and run every correctness gate"
	@echo "  make submission-check  Full verify + CLI/fuzz + evidence checks"
	@echo
	@echo "Evidence:"
	@echo "  make cli-diff          Regenerate Python/Rust CLI evidence"
	@echo "  make cli-check         Compare CLIs using temporary output"
	@echo "  make fuzz              Run 1,000 deterministic differential cases"
	@echo "  make fuzz-final-check  Re-run 5,000 cases and a 120-second session"
	@echo "  make bench-final       Generate startup/p99/RSS benchmark evidence"
	@echo "  make audit-final       Generate honest-number and safety evidence"
	@echo "  make verify-evidence   Verify committed evidence checksums/diffs"
	@echo "  make evidence-summary  Print the committed evidence headlines"
	@echo
	@echo "Container:"
	@echo "  make docker-build      Build the judge-facing verification image"
	@echo "  make docker-verify     Build and run the full Docker verification"
	@echo
	@echo "Maintenance:"
	@echo "  make clean             Remove generated virtualenv and Cargo outputs"

$(VENV_PYTHON):
	$(PYTHON) -m venv $(VENV)
	$(VENV_PYTHON) -m pip install --upgrade pip
	$(VENV_PYTHON) -m pip install -e .
	$(VENV_PYTHON) -m pip install \
		pytest pytest-mock hypothesis pytest-cov setuptools-scm tox

setup: $(VENV_PYTHON)

build:
	cargo build --manifest-path $(CARGO_MANIFEST) --release --bins

build-debug:
	cargo build --manifest-path $(CARGO_MANIFEST) --bins

fmt:
	cargo fmt --manifest-path $(CARGO_MANIFEST) --all -- --check

test-rust:
	cargo test --manifest-path $(CARGO_MANIFEST) --all-targets

clippy:
	cargo clippy --manifest-path $(CARGO_MANIFEST) --all-targets -- -D warnings

verify-tests:
	$(PYTHON) parity/verify_original_tests.py \
		--source-commit $(SOURCE_COMMIT)

test-original: setup build-debug
	$(VENV_PYTHON) parity/run_original_suite_against_rust.py -- \
		-q --continue-on-collection-errors --maxfail=0 --tb=short -ra tests

verify: verify-tests fmt test-rust clippy test-original

cli-diff: setup build
	$(VENV_PYTHON) parity/cli/run_cli_equivalence.py \
		--python-executable $(VENV_PYTHON) \
		--rust-binary rust-port/target/release/natsort

cli-check: setup build
	rm -rf /tmp/natsort-cli-check
	$(VENV_PYTHON) parity/cli/run_cli_equivalence.py \
		--python-executable $(VENV_PYTHON) \
		--rust-binary rust-port/target/release/natsort \
		--output-directory /tmp/natsort-cli-check

fuzz: setup build
	$(VENV_PYTHON) fuzz/harness.py \
		--cases 1000 \
		--seed 20260801 \
		--python-oracle $(VENV_PYTHON)

fuzz-final-check: setup build
	$(VENV_PYTHON) parity/differential_fuzz.py \
		--cases 5000 \
		--seed 20260802 \
		--python-oracle $(VENV_PYTHON)
	$(VENV_PYTHON) parity/differential_fuzz.py \
		--duration-seconds 120 \
		--seed 20260802 \
		--python-oracle $(VENV_PYTHON)

bench: setup
	$(VENV_PYTHON) parity/run_benchmarks.py \
		--python-executable $(VENV_PYTHON)

bench-final: setup build
	$(VENV_PYTHON) bench/run_judge_benchmarks.py \
		--python-executable $(VENV_PYTHON) \
		--rust-binary rust-port/target/release/natsort \
		--output-directory bench

audit-final: build
	$(PYTHON) audit/run_project_audit.py \
		--output-directory parity/evidence/audit

docker-build:
	docker build -t natsort-rust-port .

docker-verify:
	docker build -t natsort-rust-port .
	docker run --rm natsort-rust-port

verify-evidence:
	cmp -s \
		parity/test_hashes/source.sha256 \
		parity/test_hashes/submission.sha256
	test ! -s parity/evidence/cli/cli_success_output.diff
	test ! -s parity/evidence/cli/cli_output.diff
	cd fuzz && sha256sum -c checksums.sha256
	sha256sum -c parity/evidence/audit/checksums.sha256

submission-check: verify cli-check fuzz verify-evidence

evidence-summary:
	@echo "===== Original-suite integrity ====="
	@cat parity/test_hashes/verification.txt
	@echo
	@echo "===== CLI equivalence ====="
	@cat parity/evidence/cli/summary.txt
	@echo
	@echo "===== Differential fuzzing ====="
	@grep -E \
		"total_completed_cases|total_divergences|failure_artifact" \
		fuzz/log.txt
	@echo
	@echo "===== Honest numbers ====="
	@grep -E \
		"Original suite against Rust|Rust library tests|Rust Python-parity tests|Shared CLI cases|Differential fuzz cases|Differential divergences" \
		HONEST_NUMBERS.md

clean:
	rm -rf $(VENV)
	cargo clean --manifest-path $(CARGO_MANIFEST)
