SHELL := /bin/bash

.PHONY: help install dev build test preview tauri-dev tauri-build check clean

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

clean:
	rm -rf dist
	cargo clean --manifest-path src-tauri/Cargo.toml
