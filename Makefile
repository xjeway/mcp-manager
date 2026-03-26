SHELL := /bin/bash

.PHONY: help install dev build test preview tauri-dev tauri-build check clean release-current release-sync release-verify release-tag release-prepare release-publish

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
	@echo "  release-prepare VERSION=x.y.z Sync version, verify alignment, and run checks"
	@echo "  release-publish VERSION=x.y.z Verify version, create tag, and push it"
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

release-prepare:
	@test -n "$(VERSION)" || (echo "Usage: make release-prepare VERSION=x.y.z" && exit 1)
	npm run release:sync -- $(VERSION)
	npm run release:verify -- $(VERSION)
	$(MAKE) check
	@echo "Next:"
	@echo "  git add package.json package-lock.json src-tauri/tauri.conf.json src-tauri/Cargo.toml"
	@echo "  git commit -m \"chore: release v$(VERSION)\""
	@echo "  git push origin main"
	@echo "  make release-publish VERSION=$(VERSION)"

release-publish:
	@test -n "$(VERSION)" || (echo "Usage: make release-publish VERSION=x.y.z" && exit 1)
	@echo "Publishing v$(VERSION). Ensure the release commit is already pushed to main."
	npm run release:verify -- $(VERSION)
	npm run release:tag -- $(VERSION)
	git push origin v$(VERSION)

clean:
	rm -rf dist
	cargo clean --manifest-path src-tauri/Cargo.toml
