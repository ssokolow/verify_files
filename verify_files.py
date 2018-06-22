#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""A simple tool to recursively walk a set of paths and report files where
corruption was detected.

--snip--

TODO:
- Support an exclude option for the recursion
- Verify both the executable part of a .exe and potential appended archives
  (as well as trying `innoextract -t`)
- Figure out how to properly handle unzip complaining about and recovering from
  Zip files containing paths with backslashes.
- At least verify well-formedness of JSON (.json, .dashtoc) and XML
- Media files (wmv, asf, mpeg, mpg, mpe, mp2, avi, ra, ram, mkv, ogm, ogv, ogx,
    oga, mp3, webm)
- .ico (Need a custom loader to not just load the highest-quality version)
- .doc, .iso, .bin, .cue, .toc, .dll, .ttf, .otf
- Decide what to do with HTML, CSS, XUL, SVG, JS, .py, .c, and .cpp files
- I think 7-zip only verifies data.tar.gz in .debs. Verify everything.
- Use tar to check .tar.*/.tgz/etc. as a second layer of verification?
- Support an option to extract and validate nested archives
- See if this is entirely a subset of what JHOVE
  (http://jhove.openpreservation.org/) can do.
- See if I can reuse code from diffoscope
- If all else fails:
  - use Pillow to look for horizontal stripes of identical pixels in JPEGs and
    fire off a warning:
    https://www.reddit.com/r/csharp/comments/1fq46h/how_to_detect_partially_corrupt_images/
  - For images w/o CRCs, look for suspicious block-sized regions of 00 or FF.

NOTES:
- I have not yet found an automated way to detect the corruption in the
  embeddably-uncorrupt example image at https://superuser.com/q/276154

Dependencies:
- Python 3.x
- bunzip2   (for checking bzip2 files)
- gunzip    (for checking gzip files)
- p7zip     (for checking 7-Zip, .deb, .rpm, and Mobipocket files)
- pdftotext (for checking PDFs)
- Pillow    (for checking images)
- unrar     (for checking RAR/CBR/RSN files)
- unzip     (for checking Zip/CBZ/ePub/ODF/JAR/XPI files)
- unxz      (for checking .xz and .lzma files)
"""

from __future__ import (absolute_import, division, print_function,
                        with_statement, unicode_literals)

__author__ = "Stephan Sokolow (deitarion/SSokolow)"
__appname__ = "Simple recursive detector for corrupted files"
__version__ = "0.0pre0"
__license__ = "GNU GPL 3.0 or later"

import logging, os, subprocess
from PIL import Image

log = logging.getLogger(__name__)

def make_unverifiable(fmt_name):
    """Closure factory for formats with no internal integrity checks"""
    def process(path):
        """Report the given path as unverifiable"""
        log.warning("%s files cannot be verified: %s", fmt_name, path)
    return process

def make_subproc_processor(fmt_name, argv_prefix, argv_suffix=None):
    """Closure factory for formats verifiable by subprocess exit code"""
    argv_suffix = argv_suffix or []

    def process(path):
        """Verify the given path"""
        with open(os.devnull, 'w') as nul:
            try:
                subprocess.check_call(argv_prefix + [path] + argv_suffix,
                                      stdout=nul, stderr=nul)
            except subprocess.CalledProcessError:
                log.error("%s verification failed: %s", fmt_name, path)

        log.info("%s OK: %s", fmt_name, path)
    return process

def pil_processor(path):
    """Verify the given path is a valid, uncorrupted image.

    (Within the limits of the format)

    @attention: Don't use this for formats where PIL/Pillow only reads one
        of several subimages. It'll lend a false sense of security.
    """
    try:
        # Load the image to detect corruption in the most thorough way
        with open(path, 'rb') as iobj:
            img = Image.open(iobj)
            img.load()
    # TODO: Identify what Image.open can *actually* raise
    except Exception as err:  # pylint: disable=broad-except
        log.error("Image file verification failed: %s", path)
        log.debug("...because: %s: %s", err.__class__.__name__, err)

def ignore(path):
    log.debug("Ignoring %s", path)

PROCESSORS = {
    '.7z': make_subproc_processor('7-Zip', ['7z', 't']),
    '.bmp': pil_processor,
    '.bz2': make_subproc_processor('BZip2', ['bunzip2', '-t']),
    '.dcx': pil_processor,
    '.deb': make_subproc_processor('.deb', ['7z', 't']),  # TODO: Verify ALL
    '.fli': pil_processor,
    '.flc': pil_processor,
    '.gif': pil_processor,
    '.gz': make_subproc_processor('GZip', ['gunzip', '-t']),
    '.j2k': pil_processor,
    '.j2p': pil_processor,
    '.jpe': pil_processor,
    '.jpeg': pil_processor,
    '.jpg': pil_processor,  # TODO: https://superuser.com/q/276154
    '.jpx': pil_processor,
    '.lzma': make_subproc_processor('.lzma', ['unxz', '-t']),
    '.mobi': make_subproc_processor('Mobipocket', ['7z', 't']),
    '.pbm': pil_processor,
    '.pcx': pil_processor,
    '.pdf': make_subproc_processor('PDF', ['pdftotext'], ['-']),
    '.pgm': pil_processor,
    '.png': pil_processor,
    '.ppm': pil_processor,
    '.pyc': ignore,
    '.pyo': ignore,
    '.rar': make_subproc_processor('RAR', ['unrar', 't']),
    '.tga': pil_processor,
    '.tif': pil_processor,
    '.tiff': pil_processor,
    '.txt': make_unverifiable("Plaintext"),
    '.webp': pil_processor,
    '.xbm': pil_processor,
    '.xpm': pil_processor,
    '.xz': make_subproc_processor('.xz', ['unxz', '-t']),
    '.zip': make_subproc_processor('Zip', ['unzip', '-t']),
}

for alias, target in {'.tgz': '.gz', '.tbz2': '.bz2', '.txz': '.xz'}.items():
    PROCESSORS[alias] = PROCESSORS[target]
for ext in ('.cbr', '.rsn'):
    PROCESSORS[ext] = PROCESSORS['.rar']
for ext in ('.cbz', '.epub', '.jar', '.xpi',
            '.odt', '.ods', '.odp', '.odg',
            '.ott', '.ots', '.otp', '.otg'):
    PROCESSORS[ext] = PROCESSORS['.zip']

def process_file(path):
    """Check the given path for corruption"""
    fext = os.path.splitext(path)[1].lower()

    if not os.path.exists(path):
        log.error("Path does not exist: %s", path)
    elif os.stat(path).st_size == 0:
        log.error("File is empty: %s", path)
    elif fext in PROCESSORS:
        PROCESSORS[fext](path)
    else:
        log.error("Unrecognized file type: %s", path)

def walk_path(root):
    """Process the given folder tree"""
    if os.path.isfile(root):
        process_file(root)
    else:
        for path, dirs, files in os.walk(root):
            dirs.sort()

            for fname in files:
                fpath = os.path.join(path, fname)
                process_file(fpath)

def main():
    """The main entry point, compatible with setuptools entry points."""
    # If we're running on Python 2, take responsibility for preventing
    # output from causing UnicodeEncodeErrors. (Done here so it should only
    # happen when not being imported by some other program.)
    import sys
    if sys.version_info.major < 3:
        reload(sys)
        sys.setdefaultencoding('utf-8')  # pylint: disable=no-member

    from argparse import ArgumentParser, RawDescriptionHelpFormatter
    parser = ArgumentParser(formatter_class=RawDescriptionHelpFormatter,
            description=__doc__.replace('\r\n', '\n').split('\n--snip--\n')[0])
    parser.add_argument('--version', action='version',
            version="%%(prog)s v%s" % __version__)
    parser.add_argument('-v', '--verbose', action="count",
        default=2, help="Increase the verbosity. Use twice for extra effect.")
    parser.add_argument('-q', '--quiet', action="count",
        default=0, help="Decrease the verbosity. Use twice for extra effect.")
    parser.add_argument('path', nargs='+')
    # Reminder: %(default)s can be used in help strings.

    args = parser.parse_args()

    # Set up clean logging to stderr
    log_levels = [logging.CRITICAL, logging.ERROR, logging.WARNING,
                  logging.INFO, logging.DEBUG]
    args.verbose = min(args.verbose - args.quiet, len(log_levels) - 1)
    args.verbose = max(args.verbose, 0)
    logging.basicConfig(level=log_levels[args.verbose],
                        format='%(levelname)s: %(message)s')

    for path in args.path:
        walk_path(path)

if __name__ == '__main__':
    main()

# vim: set sw=4 sts=4 expandtab :
