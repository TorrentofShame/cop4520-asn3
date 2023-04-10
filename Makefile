args=$(filter-out $@, $(MAKECMDGOALS))

problem1:
	cargo b --release
	target/release/problem1 $(call args)

problem2:
	cargo b --release
	target/release/problem2 $(call args)

%:
	@:
