#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Quick script to generate corrupt test files from good ones"""

# Prevent Python 2.x PyLint from complaining if run on this
from __future__ import (absolute_import, division, print_function,
                        with_statement, unicode_literals)

__author__ = "Stephan Sokolow (deitarion/SSokolow)"
__appname__ = "Quick helper to generate draft corrupted files"
__version__ = "0.0pre0"
__license__ = "GNU GPL 3.0 or later"

import logging, os
log = logging.getLogger(__name__)


def process_args(in_dir, out_dir, offset):
    for fname in sorted(os.listdir(in_dir)):
        in_fpath = os.path.join(in_dir, fname)
        out_fpath = os.path.join(out_dir, fname)

        if os.path.exists(out_fpath):
            log.info("Already exists: %r", out_fpath)
            continue

        log.info("%r → %r", in_fpath, out_fpath)
        with open(in_fpath, 'rb') as fobj:
            data = list(fobj.read())

        data[offset] ^= 0b00000001
        data = bytes(data)

        with open(out_fpath, 'wb') as fobj:
            fobj.write(data)


def main():
    """The main entry point, compatible with setuptools entry points."""
    from argparse import ArgumentParser, RawDescriptionHelpFormatter
    parser = ArgumentParser(formatter_class=RawDescriptionHelpFormatter,
        description=__doc__.replace('\r\n', '\n').split('\n--snip--\n')[0])
    parser.add_argument('--version', action='version',
        version="%%(prog)s v%s" % __version__)
    parser.add_argument('-v', '--verbose', action="count",
        default=2, help="Increase the verbosity. Use twice for extra effect.")
    parser.add_argument('-q', '--quiet', action="count",
        default=0, help="Decrease the verbosity. Use twice for extra effect.")
    parser.add_argument('in_path', action="store",
        help="Directory to pull good files from")
    parser.add_argument('out_path', action="store",
        help="Directory to write bad files to")
    parser.add_argument('--offset', action="store", default=16, type=int,
        help="The offset of the byte to flip the lowest bit on "
        "(default: %(default)s)")

    args = parser.parse_args()

    # Set up clean logging to stderr
    log_levels = [logging.CRITICAL, logging.ERROR, logging.WARNING,
              logging.INFO, logging.DEBUG]
    args.verbose = min(args.verbose - args.quiet, len(log_levels) - 1)
    args.verbose = max(args.verbose, 0)
    logging.basicConfig(level=log_levels[args.verbose],
                format='%(levelname)s: %(message)s')

    process_args(args.in_path, args.out_path, args.offset)

if __name__ == '__main__':  # pragma: nocover
    main()

# vim: set sw=4 sts=4 expandtab :
