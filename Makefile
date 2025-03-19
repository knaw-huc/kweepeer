.PHONY: all bin docs

PREFIX := /usr

all: bin docs

bin:
	cargo build --release

install: docs
	cargo install --root $(PREFIX)
	mkdir -p $(PREFIX)/share/man/man1 $(PREFIX)/share/man/man5
	cp docs/*.1 $(PREFIX)/share/man/man1
	cp docs/*.5 $(PREFIX)/share/man/man5

docs:
	cd docs && $(MAKE) kweepeer.1 kweepeer.5

%.html: %.md
	pandoc -t html -F mermaid-filter -o $@ $<

%.pdf: %.md
	pandoc -t pdf -F mermaid-filter -o $@ $<
