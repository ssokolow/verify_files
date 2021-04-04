#!/usr/bin/env python3
good_str = "Testing 123"
bad_str = "Testing\x00123"

import os
os.chdir(os.path.dirname(os.path.abspath(__file__)))

for encoding in ('utf8', 'utf-16-le', 'utf-16-be', 'utf-32-le', 'utf-32-be'):
    with open('../good/testfile.{}.txt'.format(encoding), 'wb') as fobj:
        fobj.write(good_str.encode(encoding))
    with open('../good/testfile.{}-bom.txt'.format(encoding),
            'wb') as fobj:
        fobj.write(('\ufeff' + good_str).encode(encoding))
    with open('../bad/testfile.{}.txt'.format(encoding), 'wb') as fobj:
        fobj.write(bad_str.encode(encoding))
    with open('../bad/testfile.{}-bom.txt'.format(encoding),
            'wb') as fobj:
        fobj.write(('\ufeff' + bad_str).encode(encoding))
