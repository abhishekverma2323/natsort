SHELL := /bin/bash

PYTHON ?= python3
VENV ?= .port-venv
VENV_PYTHON := $(VENV)/bin/python
CARGO_MANIFEST := rust-port/Cargo.toml
SOURCE_COMMIT := b543bdce8771b6e7a7dae0c6745ddf7e80299797

.PHONY: help setup build build-debug fmt test-rust clippy \
        verify-tests test-original verify fuzz bench docker-build clean

help:
	@echo "make setup         Create the verification Python environment"
	@echo "make build         Build all Rust release binaries"
	@echo "make verify        Hash-check tests, run Rust gates, run 344 original tests"
	@echo "make fuzz          Run deterministic differential fuzzing"
	@echo "make bench         Run the existing in-process benchmark suite"
	@echo "make docker-build  Build the judge-facing verification image"

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

fuzz: setup build
	$(VENV_PYTHON) fuzz/harness.py \
		--cases 1000 \
		--seed 20260801 \
		--python-oracle $(VENV_PYTHON)

bench: setup
	$(VENV_PYTHON) parity/run_benchmarks.py \
		--python-executable $(VENV_PYTHON)

docker-build:
	docker build -t natsort-rust-port .

clean:
	rm -rf $(VENV)
	cargo clean --manifest-path $(CARGO_MANIFEST)
