.PHONY: server import kana-cli

server:
	@cd kotoba-server; cargo run

import:
	@cd dict-import; cargo run --release

kana-cli:
	@cd kana; cargo run --example cli
