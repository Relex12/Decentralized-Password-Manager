all: pandoc clean


tmp.md: 01-FR-comparatif.md  02-FR-solutions.md  03-FR-specifications.md  04-FR-implementation.md  05-FR-fonctionnalites.md
	for file in $^ ; do cat $$file >> $@ ; echo '\n' >> $@ ; done

tmp.html: tmp.md
	pandoc $< --output=$@


pandoc: tmp.md
	pandoc $< --output=$@.pdf --metadata-file .header.yml

wkhtmltopdf: tmp.html
	wkhtmltopdf --encoding utf-8 $< $@.pdf

weasyprint: tmp.html
	weasyprint --encoding utf-8 $< $@.pdf

mdpdf: tmp.md
	mdpdf $< $@.pdf

md-to-pdf: tmp.md
	cat $< | md-to-pdf > $@.pdf

markdown-pdf: tmp.md
	markdown-pdf $< --out $@.pdf


clean:
	rm -f tmp.*

clear: clean
	rm -f *.pdf
