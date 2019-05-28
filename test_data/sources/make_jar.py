#!/usr/bin/env python3
"""Helper script to rebuild test JARs from source"""

__author__ = "Stephan Sokolow (deitarion/SSokolow)"
__license__ = "Public Domain"

import os, shutil, subprocess, zipfile

os.chdir(os.path.dirname(os.path.abspath(__file__)))

GOOD_PATH = os.path.abspath(os.path.join(os.pardir, 'good', 'testfile.jar'))
BAD_PATH = os.path.abspath(os.path.join(os.pardir, 'bad', 'testfile.jar'))

# Compile Java code into JAR file
subprocess.check_call(['javac', 'testfile.java'])
subprocess.check_call(['jar', 'cfm', BAD_PATH,
                       'MANIFEST.MF', 'testfile.class'])

# Make a copy in bad/ and Corrupt it
shutil.copy(GOOD_PATH, BAD_PATH)
with zipfile.ZipFile(BAD_PATH) as zobj:
    offset = zobj.infolist()[1].header_offset
with open(BAD_PATH, 'rb+') as fobj:
    fobj.seek(offset)
    old_value = fobj.read(1)
    fobj.seek(offset)
    fobj.write(bytes([old_value[0] ^ 1]))

# Remove temporary file
os.remove('testfile.class')
