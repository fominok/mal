STEPS = step0_repl step1_read_print step2_eval step3_env step4_if_fn_do step5_tco step6_file step7_quote step8_macros step9_try stepA_mal

dist: mal

mal: stepA_mal
	cp $< $@

%: %.rs
	cargo build --release --bin $*
	cp target/release/$* $@

STEP0_DEPS = step0_repl.rs
STEP1_DEPS = step1_read_print.rs lexer.rs parser.rs reader.rs formatter.rs

step0_repl: $(STEP0_DEPS)
step1_read_print: $(STEP1_DEPS)

.PHONY: clean

clean:
	cargo clean
	rm -f $(STEPS)
	rm -f mal
