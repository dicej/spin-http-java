lib.wasm: target/generated/wasm/teavm-wasm/classes.wasm munge/target/debug/munge
	munge/target/debug/munge < $< > $@

target/generated/wasm/teavm-wasm/classes.wasm: src/main/java/foo/HelloSpin.java
	mvn prepare-package

munge/target/debug/munge: munge/src/main.rs
	(cd munge && cargo build)

.PHONY: clean
clean:
	rm -rf lib.wasm
	(cd munge && cargo clean)
	mvn clean
