.PHONY : lint
lint :
	cargo clippy --all-targets -- -D warnings \
		-A clippy::comparison_chain \
		-A clippy::upper-case-acronyms \
		-A dead-code
