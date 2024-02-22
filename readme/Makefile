all: pandoc clean


tmp.md: *-FR-*.md
	for file in $^ ; do cat $$file >> $@ ; echo '\n' >> $@ ; done

tmp.html: tmp.md
	pandoc $< --output=$@


pandoc: tmp.md
	pandoc $< --output=$@.pdf --metadata-file .header.yml

wkhtmltopdf: tmp.html
	wkhtmltopdf --encoding utf-8 $< $@.pdf

weasyprint: tmp.html
	weasyprint --encoding utf-8 $< $@.pdf


clean:
	rm -f tmp.*

clear: clean
	rm -f *.pdf
