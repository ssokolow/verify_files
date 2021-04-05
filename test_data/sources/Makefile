# REMINDER: For legal reasons, this makefile must use the pre-generated RAR and
#           CBR files unless you've bought and installed your own RAR
#           license key.

.PHONY: all clean

7Z_TEST=7z t >/dev/null
JSON_TEST=python3 -m json.tool >/dev/null
LSAR_TEST=lsar -t >/dev/null
TAR_TEST=tar tvaf >/dev/null
ZIP_TEST=unzip -t >/dev/null

artifacts = \
  ../good/testfile.7z \
  ../good/testfile.arc \
  ../good/testfile.arj \
  ../good/testfile.bmp \
  ../good/testfile.microsoft.cab \
  ../good/testfile.cb7 \
  ../good/testfile.cbz \
  ../good/testfile.chm \
  ../good/testfile.cramfs \
  ../good/testfile.dashtoc \
  ../good/testfile.docm \
  ../good/testfile.flac \
  ../good/testfile.gif \
  ../good/testfile.jar \
  ../good/testfile.jpe \
  ../good/testfile.jpeg \
  ../good/testfile.jpg \
  ../good/testfile.json \
  ../good/testfile.lha \
  ../good/testfile.lzh \
  ../good/testfile.lzx \
  ../good/testfile.mml \
  ../good/testfile.nsis.exe \
  ../good/testfile.ogg \
  ../good/testfile.pbm \
  ../good/testfile.pcx \
  ../good/testfile.pdf \
  ../good/testfile.pgm \
  ../good/testfile.png \
  ../good/testfile.ppm \
  ../good/testfile.ps \
  ../good/testfile.py \
  ../good/testfile.rar3.av.cbr \
  ../good/testfile.rar3.av.rar \
  ../good/testfile.rar3.cbr \
  ../good/testfile.rar3.dos_sfx.exe \
  ../good/testfile.rar3.locked.cbr \
  ../good/testfile.rar3.locked.rar \
  ../good/testfile.rar3.rar \
  ../good/testfile.rar3.rr.cbr \
  ../good/testfile.rar3.rr.rar \
  ../good/testfile.rar3.solid.cbr \
  ../good/testfile.rar3.solid.rar \
  ../good/testfile.rar3.wincon.sfx.exe \
  ../good/testfile.rar3.wingui.sfx.exe \
  ../good/testfile.rar5.cbr \
  ../good/testfile.rar5.linux_sfx.bin \
  ../good/testfile.rar5.locked.cbr \
  ../good/testfile.rar5.locked.rar \
  ../good/testfile.rar5.rar \
  ../good/testfile.rar5.rr.cbr \
  ../good/testfile.rar5.rr.rar \
  ../good/testfile.rar5.solid.cbr \
  ../good/testfile.rar5.solid.rar \
  ../good/testfile.rar5.wincon.sfx.exe \
  ../good/testfile.rar5.wingui.sfx.exe \
  ../good/testfile.rss \
  ../good/testfile.rtf \
  ../good/testfile.sqlite3 \
  ../good/testfile.gzip.squashfs \
  ../good/testfile.lz4.squashfs \
  ../good/testfile.lzma.squashfs \
  ../good/testfile.lzo.squashfs \
  ../good/testfile.xz.squashfs \
  ../good/testfile_foreign.sqlite3 \
  ../good/testfile.svg \
  ../good/testfile.svgz \
  ../good/testfile.tbz2 \
  ../good/testfile.tga \
  ../good/testfile.tgz \
  ../good/testfile.tlz \
  ../good/testfile.txt.b64 \
  ../good/testfile.txt.bz2 \
  ../good/testfile.txt.compress.Z \
  ../good/testfile.txt.gz \
  ../good/testfile.txt.hqx \
  ../good/testfile.txt.lz \
  ../good/testfile.txt.lzma \
  ../good/testfile.txt.uu \
  ../good/testfile.txt.uue \
  ../good/testfile.txt.xz \
  ../good/testfile.txt.xx \
  ../good/testfile.txt.xxe \
  ../good/testfile.txt.ync \
  ../good/testfile.txt.yenc \
  ../good/testfile.utf-16-be-bom.txt \
  ../good/testfile.utf-16-be.txt \
  ../good/testfile.utf-16-le-bom.txt \
  ../good/testfile.utf-16-le.txt \
  ../good/testfile.utf-32-be-bom.txt \
  ../good/testfile.utf-32-be.txt \
  ../good/testfile.utf-32-le-bom.txt \
  ../good/testfile.utf-32-le.txt \
  ../good/testfile.utf8-bom.txt \
  ../good/testfile.utf8.txt \
  ../good/testfile.txz \
  ../good/testfile.tif \
  ../good/testfile.tiff \
  ../good/testfile.webp \
  ../good/testfile.wim \
  ../good/testfile.xar \
  ../good/testfile.xbm \
  ../good/testfile.xlsm \
  ../good/testfile.xml \
  ../good/testfile_xml \
  ../good/testfile.xpm \
  ../good/testfile.zip \
  ../good/testfile.zoo \
  ../bad/testfile.jar

all: rar-test-files $(artifacts)

clean:
	rm -f testfile.class testfile.jpg $(artifacts)

# TODO: Add a rule to pull https://github.com/ssokolow/rar-test-files and copy
# 	the RAR and CBR files into ../good/

../good/testfile.7z: testfile.txt
	7z a -t7z -m0=lzma -mx=9 -mfb=64 -md=32m -ms=on $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	file -binNpr $@ | grep -q application/x-7z-compressed

../good/testfile.arc: testfile.txt
	arc a $@ $^
	$(LSAR_TEST) $@
	arc t $@
	file -binNpr $@ | grep -q application/x-arc

../good/testfile.arj: testfile.txt
	arj a $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	arj t $@
	file -binNpr $@ | grep -q application/x-arj

../good/testfile.txt.b64: testfile.txt
	uuenview -b $< >$@
	# TODO: Test

../good/testfile.bmp: testfile.png
	convert $< $@
	file -binNpr $@ | grep -q image/x-ms-bmp
	# TODO: Test

../good/testfile.microsoft.cab: testfile.txt
	lcab $^ $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	cabextract -t $@
	file -binNpr $@ | grep -q application/vnd.ms-cab-compressed

../good/testfile.cb7: testfile.png testfile.jpg
	7z a -t7z -m0=lzma -mx=9 -mfb=64 -md=32m -ms=on $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	file -binNpr $@ | grep -q application/x-7z-compressed

../good/testfile.cbz: testfile.png testfile.jpg
	zip -T $@ $^
	advzip -z4 $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	$(ZIP_TEST) $@
	file -binNpr $@ | grep -q application/zip

../good/testfile.chm: testfile.hhp testfile.html
	chmcmd $<
	chmod 644 $@
	$(7Z_TEST) $@
	# TODO: Assert the format more specifically

../good/testfile.cramfs: testfile.txt
	mkdir tmp
	cp $^ tmp
	mkcramfs tmp $@
	rm -rf tmp
	$(7Z_TEST) $@
	cramfsck $@ >|/dev/null 2>&1

../good/testfile.dashtoc: testfile.dashtoc
	cp $^ $@
	$(JSON_TEST) $@

../good/testfile.docm: ../good/testfile.docx
	cp $< $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	$(ZIP_TEST) $@
	# TODO: Assert the format more specifically

../good/testfile.flac: testfile.wav
	flac --best --verify -o $@ $<
	flac --test $@
	touch $@
	file -binNpr $@ | grep -q audio/x-flac

../good/testfile.gif: testfile.png
	convert $< $@
	file -binNpr $@ | grep -q image/gif

../good/testfile.jar: MANIFEST.MF testfile.class
	jar cfm $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	$(ZIP_TEST) $@
	file -binNpr $@ | grep -q application/java-archive

../good/testfile.jpe: testfile.jpg
	convert $< $@
	file -binNpr $@ | grep -q image/jpeg

../good/testfile.jpeg: testfile.png
	convert $< $@
	file -binNpr $@ | grep -q image/jpeg

../good/testfile.jpg: testfile.jpg
	cp $< $@
	file -binNpr $@ | grep -q image/jpeg

../good/testfile.json: testfile.json
	cp $^ $@
	$(JSON_TEST) $@

../good/testfile.lha: testfile.txt
	lzh-archiver c $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	file -binNpr $@ | grep -q application/x-lharc

../good/testfile.lzh: testfile.txt
	lzh-archiver c $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	file -binNpr $@ | grep -q application/x-lharc

../good/testfile.lzx: testfile.lzx
	cp $< $@
	$(LSAR_TEST) $@
	# TODO: Assert the format more specifically

../good/testfile.mml: testfile.mml
	cp $^ $@
	file -binNpr $@ | grep -q application/xml

../good/testfile.nsis.exe: testfile.nsi
	# TODO: Generate test files for a bunch of different versions.
	makensis -NOCONFIG "-XOutFile $@" $<
	$(7Z_TEST) $@

../good/testfile.ogg: testfile.wav
	oggenc -q5 -o $@ $<
	oggdec -o /dev/null 2>|/dev/null $@
	file -binNpr $@ | grep -q audio/ogg

../good/testfile.pbm: testfile.png
	convert $< $@
	file -binNpr $@ | grep -q image/x-portable-bitmap

../good/testfile.pcx: testfile.png
	convert $< $@
	file -binNpr $@ | grep -q image/x-pcx

../good/testfile.pdf: testfile.pdf
	cp $< $@
	file -binNpr $@ | grep -q application/pdf

../good/testfile.pgm: testfile.png
	convert $< $@
	file -binNpr $@ | grep -q image/x-portable-greymap

../good/testfile.png: testfile.png
	cp $< $@
	file -binNpr $@ | grep -q image/png

../good/testfile.ppm: testfile.png
	convert $< $@
	file -binNpr $@ | grep -q image/x-portable-pixmap

../good/testfile.ps: testfile.ps
	cp $< $@
	ps2ascii >|/dev/null 2>&1 $@  # Test output
	file -binNpr $@ | grep -q application/postscript

../good/testfile.py: testfile.py
	cp $< $@
	# TODO: Test

../good/testfile.rar3.av.cbr: rar-test-files/testfile.rar3.av.cbr
	cp $^ $@

../good/testfile.rar3.av.rar: rar-test-files/testfile.rar3.av.rar
	cp $^ $@

../good/testfile.rar3.cbr: rar-test-files/testfile.rar3.cbr
	cp $^ $@

../good/testfile.rar3.dos_sfx.exe: rar-test-files/testfile.rar3.dos_sfx.exe
	cp $^ $@

../good/testfile.rar3.locked.cbr: rar-test-files/testfile.rar3.locked.cbr
	cp $^ $@

../good/testfile.rar3.locked.rar: rar-test-files/testfile.rar3.locked.rar
	cp $^ $@

../good/testfile.rar3.rar: rar-test-files/testfile.rar3.rar
	cp $^ $@

../good/testfile.rar3.rr.cbr: rar-test-files/testfile.rar3.rr.cbr
	cp $^ $@

../good/testfile.rar3.rr.rar: rar-test-files/testfile.rar3.rr.rar
	cp $^ $@

../good/testfile.rar3.solid.cbr: rar-test-files/testfile.rar3.solid.cbr
	cp $^ $@

../good/testfile.rar3.solid.rar: rar-test-files/testfile.rar3.solid.rar
	cp $^ $@

../good/testfile.rar3.wincon.sfx.exe: rar-test-files/testfile.rar3.wincon.sfx.exe
	cp $^ $@

../good/testfile.rar3.wingui.sfx.exe: rar-test-files/testfile.rar3.wingui.sfx.exe
	cp $^ $@

../good/testfile.rar5.cbr: rar-test-files/testfile.rar5.cbr
	cp $^ $@

../good/testfile.rar5.linux_sfx.bin: rar-test-files/testfile.rar5.linux_sfx.bin
	cp $^ $@

../good/testfile.rar5.locked.cbr: rar-test-files/testfile.rar5.locked.cbr
	cp $^ $@

../good/testfile.rar5.locked.rar: rar-test-files/testfile.rar5.locked.rar
	cp $^ $@

../good/testfile.rar5.rar: rar-test-files/testfile.rar5.rar
	cp $^ $@

../good/testfile.rar5.rr.cbr: rar-test-files/testfile.rar5.rr.cbr
	cp $^ $@

../good/testfile.rar5.rr.rar: rar-test-files/testfile.rar5.rr.rar
	cp $^ $@

../good/testfile.rar5.solid.cbr: rar-test-files/testfile.rar5.solid.cbr
	cp $^ $@

../good/testfile.rar5.solid.rar: rar-test-files/testfile.rar5.solid.rar
	cp $^ $@

../good/testfile.rar5.wincon.sfx.exe: rar-test-files/testfile.rar5.wincon.sfx.exe
	cp $^ $@

../good/testfile.rar5.wingui.sfx.exe: rar-test-files/testfile.rar5.wingui.sfx.exe
	cp $^ $@

../good/testfile.rss: testfile.rss
	cp $< $@
	# TODO: Test

../good/testfile.rtf: testfile.rtf
	cp $< $@
	# TODO: Test

../good/testfile.sqlite3: testfile.sql
	sqlite3 $@ < $<
	sqlite3 $@ "PRAGMA integrity_check; PRAGMA foreign_key_check"

../good/testfile_foreign.sqlite3: testfile_foreign.sql
	sqlite3 $@ < $<
	sqlite3 $@ "PRAGMA integrity_check; PRAGMA foreign_key_check"

../good/testfile.gzip.squashfs: testfile.txt
	mksquashfs $^ $@ -comp gzip
	$(7Z_TEST) $@
	unsquashfs -l $@ >|/dev/null 2>&1

../good/testfile.lz4.squashfs: testfile.txt
	mksquashfs $^ $@ -comp lz4
	# $(7Z_TEST) $@  # Error: E_NOTIMPL
	unsquashfs -l $@ >|/dev/null 2>&1

../good/testfile.lzma.squashfs: testfile.txt
	mksquashfs $^ $@ -comp lzma
	$(7Z_TEST) $@
	unsquashfs -l $@ >|/dev/null 2>&1

../good/testfile.lzo.squashfs: testfile.txt
	mksquashfs $^ $@ -comp lzo -Xcompression-level 9
	$(7Z_TEST) $@
	unsquashfs -l $@ >|/dev/null 2>&1

../good/testfile.xz.squashfs: testfile.txt
	mksquashfs $^ $@ -comp xz
	# $(7Z_TEST) $@  # Error: E_NOTIMPL
	unsquashfs -l $@ >|/dev/null 2>&1

../good/testfile.svg: testfile.svg
	cp $< $@
	# TODO: Test

../good/testfile.svgz: testfile.svg
	gzip -c -9 $< > $@
	advdef -z4 $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	# TODO: Assert the format more specifically

../good/testfile.tga: testfile.png
	convert $< $@
	# TODO: Test

../good/testfile.tbz2: testfile.tar
	bzip2 -c -9 $< > $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	$(TAR_TEST) $@
	bunzip2 -c $@ | diff - testfile.tar
	file -binNpr $@ | grep -q application/x-bzip2

../good/testfile.tif: testfile.png
	convert $< $@
	# TODO: Test
	file -binNpr $@ | grep -q image/tiff

../good/testfile.tiff: testfile.png
	convert $< $@
	file -binNpr $@ | grep -q image/tiff

../good/testfile.tgz: testfile.tar
	gzip -c -9 $< > $@
	advdef -z4 $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	$(TAR_TEST) $@
	gunzip -c $@ | diff - testfile.tar
	file -binNpr $@ | grep -q application/gzip

../good/testfile.tlz: testfile.tar
	lzip -c -9 $< > $@
	$(TAR_TEST) $@
	lzip -t $@
	file -binNpr $@ | grep -q application/x-lzip

../good/testfile.txt.bz2: testfile.txt
	bzip2 -c -9 $< > $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	bunzip2 -c $@ | diff - testfile.txt
	file -binNpr $@ | grep -q application/x-bzip2

../good/testfile.txt.compress.Z: testfile.txt
	compress -V 2>&1 | grep -q ncompress  # Make sure we have the real deal
	compress -cf $< > $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	uncompress -c $@ | diff - testfile.txt
	file -binNpr $@ | grep -q application/x-compress

../good/testfile.txt.gz: testfile.txt
	gzip -c -9 $< > $@
	advdef -z4 $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	gunzip -c $@ | diff - testfile.txt
	file -binNpr $@ | grep -q application/gzip

../good/testfile.txt.hqx: testfile.txt
	binhex -d $^ > $@
	hexbin -i $@ 2>|/dev/null

../good/testfile.txt.lz: testfile.txt
	lzip -c -9 $< > $@
	lzip -t $@
	lzip -d -c $@ | diff - testfile.txt
	file -binNpr $@ | grep -q application/x-lzip

../good/testfile.txt.lzma: testfile.txt
	lzma -c -9 $< > $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	unlzma -c $@ | diff - testfile.txt
	file -binNpr $@ | grep -q application/x-lzma

../good/testfile.txt.xz: testfile.txt
	xz -c -9 $< > $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	unxz -c $@ | diff - testfile.txt
	file -binNpr $@ | grep -q application/x-xz

../good/testfile.txz: testfile.tar
	xz -c -9 $< > $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	$(TAR_TEST) $@
	unxz -c $@ | diff - testfile.tar
	file -binNpr $@ | grep -q application/x-xz

../good/testfile.txt.uu: testfile.txt
	uuenview -u $< >$@
	# TODO: Test

../good/testfile.txt.uue: ../good/testfile.txt.uu
	cp $< $@
	# TODO: Test

../good/testfile.utf-16-be-bom.txt ../good/testfile.utf-16-be.txt ../good/testfile.utf-16-le-bom.txt ../good/testfile.utf-16-le.txt ../good/testfile.utf-32-be-bom.txt ../good/testfile.utf-32-be.txt ../good/testfile.utf-32-le-bom.txt ../good/testfile.utf-32-le.txt ../good/testfile.utf8-bom.txt ../good/testfile.utf8.txt: make_test_txt.py
	python3 make_test_txt.py

../good/testfile.webp: testfile.png
	convert $< $@
	# TODO: Test
	file -binNpr $@ | grep -q image/webp

../good/testfile.wim: testfile.txt
	7z a $@ $^
	$(7Z_TEST) $@
	# TODO: Assert the format more specifically

../good/testfile.xar: testfile.txt
	xar -c -f $@ $^
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	xar -t -f $@ $^

../good/testfile.xbm: testfile.png
	convert $< $@
	# TODO: Test

../good/testfile.xlsm: ../good/testfile.xlsx
	cp $< $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	$(ZIP_TEST) $@
	# TODO: Assert the format more specifically

../good/testfile.xml: testfile.xml
	cp $< $@
	# TODO: Test

# Extensionless XML test
../good/testfile_xml: testfile.xml
	cp $< $@
	# TODO: Test

../good/testfile.xpm: testfile.png
	convert $< $@
	# TODO: Test
	file -binNpr $@ | grep -q image/x-xpmi

../good/testfile.txt.xx: testfile.txt
	uuenview -x $< >$@
	# TODO: Test

../good/testfile.txt.xxe: ../good/testfile.txt.xx
	cp $< $@
	# TODO: Test

../good/testfile.txt.ync: testfile.txt
	uuenview -y $< >$@
	# TODO: Test

../good/testfile.txt.yenc: ../good/testfile.txt.ync
	cp $< $@
	# TODO: Test

../good/testfile.zip: testfile.txt
	zip -T $@ $^
	advzip -z4 $@
	$(7Z_TEST) $@
	$(LSAR_TEST) $@
	$(ZIP_TEST) $@
	file -binNpr $@ | grep -q application/zip

../good/testfile.zoo: testfile.txt
	zoo ah $@ $^
	touch $@
	$(LSAR_TEST) $@
	file -binNpr $@ | grep -q application/x-zoo

../bad/testfile.jar: ../good/testfile.jar
	# TODO: Rework corrupt_zip.py to take an output *directory* so I can
	# use $^ instead of $<
	python3 corrupt_zip.py $< $@
	# TODO: Test that extractors report FAILURE

rar-test-files:
	git clone https://github.com/ssokolow/rar-test-files.git

testfile.class: testfile.java
	javac $^

testfile.jpg: testfile.png
	convert $< $@

testfile.tar: testfile.txt
	tar cvf $@ $<
