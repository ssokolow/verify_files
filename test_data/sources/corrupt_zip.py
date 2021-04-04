#!/usr/bin/env python3
"""Helper script to introduce corruption into Zip files"""

# Prevent Python 2.x PyLint from complaining if run on this
from __future__ import (absolute_import, division, print_function,
                        with_statement, unicode_literals)

__author__ = "Stephan Sokolow (deitarion/SSokolow)"
__license__ = "Public Domain"

import os, shutil, zipfile


def main():
    """The main entry point, compatible with setuptools entry points."""
    from argparse import ArgumentParser, RawDescriptionHelpFormatter
    parser = ArgumentParser(formatter_class=RawDescriptionHelpFormatter,
        description=__doc__.replace('\r\n', '\n').split('\n--snip--\n')[0])
    parser.add_argument('in_path', action="store", help="File to read")
    parser.add_argument('out_path', action="store", help="File to write")
    # Reminder: %(default)s can be used in help strings.

    args = parser.parse_args()

    # Make a copy in bad/ and Corrupt it
    shutil.copy(args.in_path, args.out_path)
    with zipfile.ZipFile(args.out_path) as zobj:
        offset = zobj.infolist()[1].header_offset
    with open(args.out_path, 'rb+') as fobj:
        fobj.seek(offset)
        old_value = fobj.read(1)
        fobj.seek(offset)
        fobj.write(bytes([old_value[0] ^ 1]))

if __name__ == '__main__':  # pragma: nocover
    main()

# vim: set sw=4 sts=4 expandtab :
