%.html: %.md
	pandoc -t html -F mermaid-filter -o $@ $<

%.pdf: %.md
	pandoc -t pdf -F mermaid-filter -o $@ $<


