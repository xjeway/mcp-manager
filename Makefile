SHELL := /bin/bash

.PHONY: help install dev build test preview tauri-dev tauri-build check clean release-current release-sync release-verify release-tag

help:
	@echo "Available targets:"
	@echo "  install      Install npm dependencies"
	@echo "  dev          Run frontend dev server"
	@echo "  build        Build frontend"
	@echo "  test         Run unit tests"
	@echo "  preview      Preview built frontend"
	@echo "  tauri-dev    Run Tauri desktop app in dev mode"
	@echo "  tauri-build  Build Tauri desktop app"
	@echo "  check        Run frontend build + tests + Rust check"
	@echo "  release-current Show aligned release versions"
	@echo "  release-sync VERSION=x.y.z   Sync release version across project files"
	@echo "  release-verify VERSION=x.y.z Verify release version alignment"
	@echo "  release-tag VERSION=x.y.z    Create annotated git tag for release"
	@echo "  clean        Remove dist and Rust target artifacts"

install:
	npm install

dev:
	npm run dev

build:
	npm run build

test:
	npm test

preview:
	npm run preview

tauri-dev:
	npm run tauri -- dev

tauri-build:
	npm run tauri -- build

check: build test
	cargo check --manifest-path src-tauri/Cargo.toml

release-current:
	npm run release:current

release-sync:
	@test -n "$(VERSION)" || (echo "Usage: make release-sync VERSION=x.y.z" && exit 1)
	npm run release:sync -- $(VERSION)

release-verify:
	@test -n "$(VERSION)" || (echo "Usage: make release-verify VERSION=x.y.z" && exit 1)
	npm run release:verify -- $(VERSION)

release-tag:
	@test -n "$(VERSION)" || (echo "Usage: make release-tag VERSION=x.y.z" && exit 1)
	npm run release:tag -- $(VERSION)

clean:
	rm -rf dist
	cargo clean --manifest-path src-tauri/Cargo.toml
