PROJECT=runner

.PHONY: install
install: src/*.rs
	. ./config.env && cargo install --path .
	TMPDIR=$$(mktemp -d) bash -c '\
			trap "launchctl remove $(PROJECT)_tmp" EXIT; \
			launchctl submit -l $(PROJECT)_tmp -o "$${TMPDIR}"/out.txt -e "$${TMPDIR}"/err.txt \
				-- ~/.cargo/bin/$(PROJECT) _ \
				ls ~/Desktop ~/Downloads ~/Documents; \
			until test -s "$${TMPDIR}"/out.txt; do sleep 0.1; done; \
			'

test:
	cargo test -- --test-threads=1
