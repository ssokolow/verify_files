.PHONY: all clean

7Z_TEST=7z t >/dev/null
JSON_TEST=python3 -m json.tool >/dev/null
LSAR_TEST=lsar -t >/dev/null
ZIP_TEST=unzip -t >/dev/null

artifacts = \
  ../good/testfile.7z \
  ../good/testfile.arc \
  ../good/testfile.arj \
  ../good/testfile.bmp \
  ../good/testfile.microsoft.cab \
  ../good/testfile.chm \
  ../good/testfile.dashtoc \
  ../good/testfile.gif \
  ../good/testfile.jar \
  ../good/testfile.jpe \
  ../good/testfile.jpeg \
  ../good/testfile.jpg \
  ../good/testfile.json \
  ../good/testfile.lha \
  ../good/testfile.lzh \
  ../good/testfile.mml \
  ../good/testfile.pbm \
  ../good/testfile.pcx \
  ../good/testfile.pgm \
  ../good/testfile.png \
  ../good/testfile.ps \
  ../good/testfile.py \
  ../good/testfile.rss \
  ../good/testfile.rtf \
  ../good/testfile.tga \
  ../good/testfile.tif \
  ../good/testfile.tiff \
  ../good/testfile.wim \
  ../good/testfile.xar \
  ../good/testfile.xbm \
  ../good/testfile.xpm \
  ../good/testfile.zoo \
  ../good/testfile.txt.bz2 \
  ../good/testfile.txt.compress.Z \
  ../good/testfile.txt.gz \
  ../bad/testfile.jar

all: $(artifacts)

clean: 
	rm -f testfile.class $(artifacts)

# TODO: Add a rule to pull https://github.com/ssokolow/rar-test-files and copy
# 	the RAR and CBR files into ../good/

../good/testfile.7z: testfile.txt
	7z a -t7z -m0=lzma -mx=9 -mfb=64 -md=32m -ms=on $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@

../good/testfile.arc: testfile.txt
	arc a $@ $^
	$(LSAR_TEST) $@

../good/testfile.arj: testfile.txt
	arj a $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@

../good/testfile.bmp: testfile.png
	convert $< $@

../good/testfile.microsoft.cab: testfile.txt
	lcab $^ $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@

../good/testfile.chm: testfile.hhp testfile.html
	chmcmd $<
	chmod 644 $@
	$(7Z_TEST) $@

../good/testfile.dashtoc: testfile.dashtoc
	cp $^ $@
	$(JSON_TEST) $@

../good/testfile.gif: testfile.png
	convert $< $@

../good/testfile.jar: MANIFEST.MF testfile.class
	jar cfm $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	$(ZIP_TEST) $@

../good/testfile.jpe: testfile.png
	convert $< $@

../good/testfile.jpeg: testfile.png
	convert $< $@

../good/testfile.jpg: testfile.png
	convert $< $@

../good/testfile.json: testfile.json
	cp $^ $@
	$(JSON_TEST) $@

../good/testfile.lha: testfile.txt
	lzh-archiver c $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@

../good/testfile.lzh: testfile.txt
	lzh-archiver c $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@

../good/testfile.mml: testfile.mml
	cp $^ $@

../good/testfile.pbm: testfile.png
	convert $< $@

../good/testfile.pcx: testfile.png
	convert $< $@

../good/testfile.pgm: testfile.png
	convert $< $@

../good/testfile.png: testfile.png
	cp $< $@

../good/testfile.ps: testfile.ps
	cp $< $@

../good/testfile.py: testfile.py
	cp $< $@

../good/testfile.rss: testfile.rss
	cp $< $@

../good/testfile.rtf: testfile.rtf
	cp $< $@

../good/testfile.tga: testfile.png
	convert $< $@

../good/testfile.tif: testfile.png
	convert $< $@

../good/testfile.tiff: testfile.png
	convert $< $@

../good/testfile.wim: testfile.txt
	7z a $@ $^
	$(7Z_TEST) $@

../good/testfile.xar: testfile.txt
	xar -c -f $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@

../good/testfile.xbm: testfile.png
	convert $< $@

../good/testfile.xpm: testfile.png
	convert $< $@

../good/testfile.zoo: testfile.txt
	zoo ah $@ $^
	touch $@
	$(LSAR_TEST) $@

../good/testfile.txt.bz2: testfile.txt
	bzip2 -c -9 $< > $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@

../good/testfile.txt.compress.Z: testfile.txt
	compress -V 2>&1 | grep -q ncompress  # Make sure we have the real deal
	compress -cf $< > $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@

../good/testfile.txt.gz: testfile.txt
	gzip -c -9 $< > $@
	advdef -z4 $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@

../bad/testfile.jar: ../good/testfile.jar
	# TODO: Rework corrupt_zip.py to take an output *directory* so I can
	# use $^ instead of $<
	python3 corrupt_zip.py $< $@
	# TODO: Test that extractors report FAILURE

testfile.class: testfile.java
	javac $^
