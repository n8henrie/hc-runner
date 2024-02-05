PROJECT=hc-runner

.PHONY: install
install: src/*.rs
	cargo install --path .

# Please see the `macOS` section of `README.md` for the background on this
# target
.PHONY: install-macos
install-macos: install
	TMPDIR=$$(mktemp -d) bash -c '\
			trap "launchctl bootout gui/$${UID}/$(PROJECT)_tmp" EXIT; \
			launchctl submit -l come.n8henrie.$(PROJECT)_tmp -o "$${TMPDIR}"/out.txt -e "$${TMPDIR}"/err.txt \
				-- ~/.cargo/bin/$(PROJECT) \
				--slug runner-rs-setup-delete-me \
				--url http://fake \
				ls ~/Desktop ~/Downloads ~/Documents \
				|| true; \
			until test -s "$${TMPDIR}"/out.txt; do sleep 0.1; done; \
			'

.PHONY: clean
clean:
	cargo clean

.PHONY: test
test:
	cargo test -- --test-threads=1

.PHONY: lint
lint:
	cargo clippy --all-targets --all-features --workspace -- --warn clippy::pedantic
