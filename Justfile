set dotenv-load := false

default:
    @just --list

setup:
    cargo xtask setup

doctor:
    cargo xtask doctor

fmt:
    cargo xtask fmt

fmt-check:
    cargo xtask fmt-check

check:
    cargo xtask check

check-cranelift:
    cargo xtask check-cranelift

clippy:
    cargo xtask clippy

test:
    cargo xtask nextest

test-ci:
    cargo xtask nextest-ci

test-compat:
    cargo xtask test

ci:
    cargo xtask ci

verify:
    cargo xtask verify

audit-docs:
    cargo xtask audit-docs

file-size:
    cargo xtask file-size

deny:
    cargo xtask deny

secrets:
    cargo xtask secrets

hooks:
    cargo xtask hooks

pre-commit:
    cargo xtask pre-commit

pre-push:
    cargo xtask pre-push
