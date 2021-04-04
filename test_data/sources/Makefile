.PHONY: all clean

all: ../good/testfile.chm ../good/testfile.jar ../bad/testfile.jar

clean: 
	rm -f testfile.class

../good/testfile.chm: testfile.html testfile.hhp
	chmcmd testfile.hhp
	chmod 644 ../good/testfile.chm

../good/testfile.jar: MANIFEST.MF testfile.class
	jar cfm $@ $^

../bad/testfile.jar: ../good/testfile.jar
	python3 corrupt_zip.py $< $@

testfile.class: testfile.java
	javac $^
