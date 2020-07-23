.PHONY: server import-data build-data kana-cli

server:
	@cd kotoba-server; cargo run

import-data:
	@cd dict-import; cargo run --release

build-data:
	@cd dict-build; cargo run --release

kana-cli:
	@cd kana; cargo run --example cli
