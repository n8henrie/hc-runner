include config.env
export

PROJECT=runner

.PHONY: install
install: src/*.rs
	cargo install --path .

.PHONY: install-macos
install-macos: install
	TMPDIR=$$(mktemp -d) bash -c '\
			trap "launchctl bootout gui/$${UID}/$(PROJECT)_tmp" EXIT; \
			launchctl submit -l $(PROJECT)_tmp -o "$${TMPDIR}"/out.txt -e "$${TMPDIR}"/err.txt \
				-- ~/.cargo/bin/$(PROJECT) runner-rs-setup-delete-me \
				ls ~/Desktop ~/Downloads ~/Documents; \
			until test -s "$${TMPDIR}"/out.txt; do sleep 0.1; done; \
			'

.PHONY: clean
clean:
	cargo clean

.PHONY: test
test:
	cargo test --features=mocks -- --test-threads=1
