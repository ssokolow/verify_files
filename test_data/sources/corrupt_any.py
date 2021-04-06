#!/usr/bin/env python3
"""Helper script to introduce corruption into generic files"""

# Prevent Python 2.x PyLint from complaining if run on this
from __future__ import (absolute_import, division, print_function,
                        with_statement, unicode_literals)

__author__ = "Stephan Sokolow (deitarion/SSokolow)"
__license__ = "Public Domain"

import os, re, shlex, shutil, struct, sys, zipfile
import subprocess  # nosec

EXTRACTORS = [
    ['7z', 't'],
    ['lsar', '-t'],
    ['unzip', '-tqq'],
]


def main():
    """The main entry point, compatible with setuptools entry points."""
    from argparse import ArgumentParser, RawDescriptionHelpFormatter
    parser = ArgumentParser(formatter_class=RawDescriptionHelpFormatter,
        description=__doc__.replace('\r\n', '\n').split('\n--snip--\n')[0])
    parser.add_argument('in_path', action="store", help="File to read")
    parser.add_argument('out_path', action="store", help="File to write")
    parser.add_argument(
        '-o', '--offset', action="store", type=int, default=None,
        help="Set offset to corrupt (default is middle of file)")
    parser.add_argument('-c', '--test-cmd', action="store",
        help="Command to run which should pass before and fail after")
    parser.add_argument('-m', '--fail-msg', action="store",
        help="String which must also occur in the failure message")
    # Reminder: %(default)s can be used in help strings.

    args = parser.parse_args()

    if not os.path.exists(args.in_path):
        print("ERROR: Path does not exist: ", args.in_path)
        sys.exit(1)

    # Test that the output is NOT detected as corrupt
    if args.test_cmd:
        test_cmd = shlex.split(args.test_cmd)
        subprocess.check_call(test_cmd + [args.in_path],  # nosec
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL)

    if args.fail_msg:
        fail_msg = args.fail_msg.encode('utf8')
    else:
        fail_msg = None

    # Make a copy in bad/ and Corrupt it
    shutil.copy(args.in_path, args.out_path)

    if args.offset:
        offset = args.offset
    else:
        file_len = os.stat(args.out_path).st_size
        offset = file_len // 2

    with open(args.out_path, 'rb+') as fobj:
        # Seek to the file name length field in the local file header
        fobj.seek(offset)
        old_value = fobj.read(1)
        fobj.seek(offset)
        fobj.write(bytes([old_value[0] ^ 1]))

    # Test that the output is detected as corrupt
    if test_cmd:
        try:
            out = subprocess.check_output(test_cmd + [args.out_path],
                stderr=subprocess.STDOUT)  # nosec
            #print(out)
            print("Did not error:", args.out_path)
            sys.exit(2)
        except subprocess.CalledProcessError as err:
            if fail_msg and not re.search(fail_msg, err.output):
                print(err.output)
                print("Failed message check")
                sys.exit(3)
            else:
                pass  # print(err.output)


if __name__ == '__main__':  # pragma: nocover
    main()

# vim: set sw=4 sts=4 expandtab :
