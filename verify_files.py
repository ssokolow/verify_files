#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""A simple tool to recursively walk a set of paths and report files where
corruption was detected.

--snip--

TODO:
- Support an exclude option for the recursion
- Use `from distutils.spawn import find_executable as which` to detect missing
  dependencies and give a more helpful error message
- Verify .git/ repositories using `git fsck`
- Verify both the executable part of a .exe and potential appended archives
  (as well as trying `innoextract -t`)
- Figure out how to properly handle unzip complaining about and recovering from
  Zip files containing paths with backslashes.
- Verify well-formedness of JSON (.json, .dashtoc)
- Use `defusedxml` to verify the well-formedness of XML files
- Media files (wmv, asf, mpeg, mpg, mpe, mp2, avi, ra, ram, mkv, ogm, ogv, ogx,
    oga, mp3, webm)
  - https://superuser.com/q/100288/48014
- .ico (Need a custom loader to not just load the highest-quality version)
- .doc, .iso, .bin, .cue, .toc, .dll, .ttf, .otf, .djvu, .chm, .ps, .mobi, .prc
  - .doc will require a magic number check to differentiate MS Word documents
    from DOS-era text files with .DOC extensions.
- Decide what to do with HTML, CSS, XUL, SVG, JS, .py, .c, .cpp, and .asm files
- Check whether any of the tools listed at
  https://unix.stackexchange.com/a/312356/28019 can be pressed into service
  for checking for corruption in RTF files and, if so, which is best.
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
- Info-ZIP is used rather than Python's built-in Zip support

Dependencies:
- Python 3.x
- p7zip     (for checking 7-Zip, .deb, .rpm, and Mobipocket files)
- pdftotext (for checking PDFs)
- Pillow    (for checking images)
- unrar     (for checking RAR/CBR/RSN files)
"""

from __future__ import (absolute_import, division, print_function,
                        with_statement, unicode_literals)

__author__ = "Stephan Sokolow (deitarion/SSokolow)"
__appname__ = "Simple recursive detector for corrupted files"
__version__ = "0.0pre0"
__license__ = "GNU GPL 3.0 or later"

import logging, os, subprocess
import ast, bz2, gzip, json, lzma, sqlite3, tarfile, zipfile
from distutils.spawn import find_executable as which

try:
    from PIL import Image
except ImportError:
    Image = None

try:
    from defusedxml.sax import parse as sax_parse
    from xml.sax import ContentHandler as SAX_ContentHandler
except ImportError:
    sax_parse = None

log = logging.getLogger(__name__)

def read_chunked(file_obj, chunk_size=65535):
    """Generator to tidy up code that reads a file incrementally"""
    for block in iter(lambda: file_obj.read(chunk_size), b''):
        yield block

def ignore(path):
    """Ignore the given path, logging a debug-level message."""
    log.debug("Ignoring %s", path)

def pil_multi_processor(path):
    """Do an incomplete check of multi-frame image file validity"""
    if pil_processor(path):
        log.warning("Support for verifying all frames in image file not yet "
                    "implemented: %s", path)

def json_processor(path):
    """Do a basic well-formedness check on the given JSON file"""
    try:
        with open(path, 'r') as fobj:
            json.load(fobj)
    except Exception as err:  # pylint: disable=broad-except
        # I've seen TypeError but I'm not sure if that's the only type possible
        log.error("JSON file is not well-formed: %s", path)
        log.debug("...because: %s: %s", err.__class__.__name__, err)
    else:
        log.info("JSON file is well-formed: %s", path)

def pdf_processor(path):
    """Use pdftotext to check the validity of a PDF file as well as possible"""
    try:
        result = subprocess.check_output(['pdftotext', path, os.devnull],
                                         stderr=subprocess.STDOUT).strip()
        if b'Error' in result:
            log.error("PDF file is not well-formed: %s", path)
            log.debug("...because:\n%s", result)
            return
    except subprocess.CalledProcessError as err:
        log.error("PDF file is not well-formed: %s", path)
        log.debug("...because: %s: %s", err.__class__.__name__, err)
        return

    log.info("PDF file is well-formed: %s", path)

def pil_processor(path):
    """Verify the given path is a valid, uncorrupted image.

    (Within the limits of the format)

    @attention: Don't use this for formats where PIL/Pillow only reads one
        of several subimages. It'll lend a false sense of security.
    """
    if Image is None:
        log.warning("Must install Pillow to test image files: %s", path)
        return False

    try:
        # Load the image to detect corruption in the most thorough way
        with open(path, 'rb') as iobj:
            img = Image.open(iobj)
            img.load()
        # TODO: rework so this can display an OK message
        return True
    # TODO: Identify what Image.open can *actually* raise
    except Exception as err:  # pylint: disable=broad-except
        log.error("Image file verification failed: %s", path)
        log.debug("...because: %s: %s", err.__class__.__name__, err)
        return False

def py_processor(path):
    """Do a basic SyntaxError check on the given Python file"""
    try:
        with open(path, 'rb') as fobj:
            ast.parse(fobj.read())
    except Exception as err:  # pylint: disable=broad-except
        # I've seen TypeError (null bytes) and SyntaxError but I'm not sure
        # if those are the only types of error possible
        log.error("Python file is not syntactically valid: %s", path)
        log.debug("...because: %s: %s", err.__class__.__name__, err)
        return

    log.info("Python file is syntactically valid: %s", path)

def tar_processor(path):
    """Verify the given tar archive's integrity to whatever extent possible

    (Unless compressed with something like gzip, tar files have no CRCs)
    """
    try:
        with tarfile.open(path) as tobj:
            tobj.getmembers()
    except Exception as err:
        log.error("TAR verification failed: %s", path)
        log.debug("...because: %s: %s", err.__class__.__name__, err)
    else:
        log.info("TAR OK: %s", path)

def make_compressed_processor(fmt_name, module):
    """Closure factory for module.open().read()-based verifiers"""
    def check(path):
        """Check whether the given file has a valid CRC"""
        try:
            with module.open(path, mode='rb') as fobj:
                for _block in read_chunked(fobj):
                    pass  # Make sure the decompressor has to CRC everything
        except Exception as err:
            log.error("%s verification failed: %s", fmt_name, path)
            log.debug("...because: %s: %s", err.__class__.__name__, err)
        else:
            log.info("%s OK: %s", fmt_name, path)
    return check

def make_header_check(magic_num_str):
    """Closure factory for 'file has magic number' checks"""
    def check(path):
        """Check whether the specified file has the predefined magic number"""
        with open(path, 'rb') as fobj:
            return fobj.read(len(magic_num_str)) == magic_num_str

    return check

def make_subproc_processor(fmt_name, argv_prefix, argv_suffix=None):
    """Closure factory for formats verifiable by subprocess exit code"""
    argv_suffix = argv_suffix or []

    def process(path):
        """Verify the given path"""
        if not which(argv_prefix[0]):
            log.warning("Must install %s to test %s files",
                        argv_prefix[0], fmt_name)
            return

        with open(os.devnull, 'w') as nul:
            try:
                subprocess.check_call(argv_prefix + [path] + argv_suffix,
                                      stdout=nul, stderr=nul)
            except subprocess.CalledProcessError:
                log.error("%s verification failed: %s", fmt_name, path)
                return

        log.info("%s OK: %s", fmt_name, path)
    return process

def make_unverifiable(fmt_name):
    """Closure factory for formats with no internal integrity checks"""
    def process(path):
        """Do a simple read check, but then report the path as unverifiable"""
        try:
            with open(path, 'rb') as fobj:
                for _block in read_chunked(fobj):
                    pass  # Just verify that the file contents are readable
        except (OSError, IOError):
            log.error("Error while reading file: %s", path)
        else:
            log.warning("%s files cannot be verified: %s", fmt_name, path)
    return process

def make_zip_processor(fmt_name):
    """Closure factory for formats which use Zip archives as containers"""
    def process(path):
        """Check the Zip file for CRC errors"""
        try:
            with zipfile.ZipFile(path) as zobj:
                first_bad = zobj.testzip()
                if first_bad:
                    log.error("Error encountered in %r at %r", path, first_bad)
                    return
        except Exception as err:
            log.error("%s verification failed: %s", fmt_name, path)
            log.debug("...because: %s: %s", err.__class__.__name__, err)
            return

        log.info("%s OK: %s", fmt_name, path)
    return process

def sqlite3_processor(path):
    """Run SQLite 3.x's 'PRAGMA integrity_check' on a database"""
    try:
        sqlite3.connect(path).execute('PRAGMA integrity_check')
    except Exception as err:
        log.error("SQLite 3.x database verification failed: %s", path)
        log.debug("...because: %s: %s", err.__class__.__name__, err)
        return

    log.info("SQLite 3.x database passes integrity check: %s", path)

def xml_processor(path):
    """Do a basic well-formedness check on the given XML file"""
    if sax_parse is None:
        log.warning("Must install defusedxml to test XML files: %s", path)
        return

    try:
        with open(path, 'rb') as fobj:
            # We just want the side-effect of throwing an exception if the XML
            # couldn't be parsed
            sax_parse(fobj, SAX_ContentHandler())
    except Exception as err:
        log.error("XML is not well-formed: %s", path)
        log.debug("...because: %s: %s", err.__class__.__name__, err)
        return

    log.info("XML is well-formed: %s", path)

ffmpeg_cmd = ['ffmpeg', '-f', 'null', '-', '-i']

# TODO: Rework so things like .tar.gz can be checked as compressed tar files
#       rather than just .gz files. (Probably best to just use precedence-based
#       header checks but with an "expected extensions" list for each format.
EXT_PROCESSORS = {
    '.3gp': make_subproc_processor('MPEG-4 Part 12 Media', ffmpeg_cmd),
    '.3g2': make_subproc_processor('MPEG-4 Part 12 Media', ffmpeg_cmd),
    '.7z': make_subproc_processor('7-Zip archive', ['7z', 't']),
    '.aac': make_subproc_processor('AAC (ADTS Stream)', ffmpeg_cmd),
    '.ape': make_subproc_processor('Monkey\'s Audio', ffmpeg_cmd),
    '.arj': make_subproc_processor('ARJ archive', ['arj', 't']),
    '.aif': make_subproc_processor('AIFF Audio', ffmpeg_cmd),
    '.aifc': make_subproc_processor('AIFF Audio (Compressed)', ffmpeg_cmd),
    '.aiff': make_subproc_processor('AIFF Audio', ffmpeg_cmd),
    '.asf': make_subproc_processor('Microsoft ASF', ffmpeg_cmd),
    '.avi': make_subproc_processor('Microsoft AVI Video', ffmpeg_cmd),
    '.bmp': pil_processor,
    '.bz2': make_compressed_processor('BZip2', bz2),
    '.cb7': make_subproc_processor('Comic Book Archive (7-Zip)', ['7z', 't']),
    '.cbz': make_zip_processor('Comic Book Archive (Zip)'),
    '.cur': pil_multi_processor,
    '.dashtoc': json_processor,
    '.dcx': pil_multi_processor,
    '.deb': make_subproc_processor('.deb', ['7z', 't']),
    '.dib': pil_processor,
    '.docm': make_zip_processor('Macro-enabled OOXML Document'),
    '.docx': make_zip_processor('OOXML Document'),
    '.epub': make_zip_processor('ePub e-book'),
    '.flac': make_subproc_processor('FLAC', ['flac', '-t']),
    '.f4a': make_subproc_processor('FLV Audio', ffmpeg_cmd),
    '.f4b': make_subproc_processor('FLV Audiobook', ffmpeg_cmd),
    '.f4v': make_subproc_processor('FLV Video', ffmpeg_cmd),
    '.fli': pil_processor,
    '.flc': pil_processor,
    '.flv': make_subproc_processor('Flash Video', ffmpeg_cmd),
    '.gif': pil_processor,
    '.gz': make_compressed_processor('GZip', gzip),
    '.ico': pil_multi_processor,
    # TODO: Do a well-formedness check on HTML and report it as unverifiable
    #       on success, since there's no way to catch corruption in the text.
    '.j2k': pil_processor,
    '.jar': make_zip_processor('Java ARchive'),
    '.jfi': pil_processor,
    '.jfif': pil_processor,
    '.jif': pil_processor,
    '.jpe': pil_processor,
    '.jpeg': pil_processor,
    '.jpg': pil_processor,
    '.jp2': pil_processor,
    '.jpf': pil_processor,
    '.jpx': pil_processor,
    '.json': json_processor,
    '.lz': make_subproc_processor('Lzip', ['lzip', '-t']),
    '.lzma': make_compressed_processor('.lzma', lzma),
    '.m4a': make_subproc_processor('MPEG-4 Part 14 Audio', ffmpeg_cmd),
    '.m4b': make_subproc_processor('MPEG-4 Part 14 Audiobook', ffmpeg_cmd),
    '.m4r': make_subproc_processor('MPEG-4 Part 14 Ringtone', ffmpeg_cmd),
    '.m4v': make_subproc_processor('MPEG-4 Part 14 Video', ffmpeg_cmd),
    '.mk3d': make_subproc_processor('Matroska Video (3D)', ffmpeg_cmd),
    '.mka': make_subproc_processor('Matroska Audio', ffmpeg_cmd),
    '.mkv': make_subproc_processor('Matroska Video', ffmpeg_cmd),
    '.mov': make_subproc_processor('Quicktime Video', ffmpeg_cmd),
    '.mp+': make_subproc_processor('Musepack Audio', ffmpeg_cmd),
    '.mp1': make_subproc_processor('MPEG Layer 1 Audio', ffmpeg_cmd),
    '.mp2': make_subproc_processor('MPEG Layer 2 Audio', ffmpeg_cmd),
    '.mp3': make_subproc_processor('MPEG Layer 3 Audio', ffmpeg_cmd),
    '.mp4': make_subproc_processor('MPEG-4 Part 14 Video', ffmpeg_cmd),
    '.mpc': make_subproc_processor('Musepack Audio', ffmpeg_cmd),
    '.mpe': make_subproc_processor('MPEG Video', ffmpeg_cmd),
    '.mpeg': make_subproc_processor('MPEG Video', ffmpeg_cmd),
    '.mpg': make_subproc_processor('MPEG Video', ffmpeg_cmd),
    '.mpp': make_subproc_processor('Musepack Audio', ffmpeg_cmd),
    '.odg': make_zip_processor('ODF Drawing'),
    '.odp': make_zip_processor('ODF Presentation'),
    '.ods': make_zip_processor('ODF Spreadsheet'),
    '.odt': make_zip_processor('ODF Text Document'),
    '.oga': make_subproc_processor('Ogg containing audio (.oga)', ffmpeg_cmd),
    '.ogg': make_subproc_processor('Ogg Vorbis (.ogg)', ffmpeg_cmd),
    '.ogm': make_subproc_processor('OGM (Ogg, Unofficial)', ffmpeg_cmd),
    '.ogv': make_subproc_processor('Ogg containing video (.ogv)', ffmpeg_cmd),
    '.ogx': make_subproc_processor('Ogg (unspecified) (.ogx)', ffmpeg_cmd),
    '.opus': make_subproc_processor('Opus Audio', ffmpeg_cmd),
    '.otg': make_zip_processor('ODF Drawing Template'),
    '.otp': make_zip_processor('ODF Presentation Template'),
    '.ots': make_zip_processor('ODF Spreadsheet Template'),
    '.ott': make_zip_processor('ODF Text Document Template'),
    '.pbm': pil_processor,
    '.pcx': pil_processor,
    '.pdf': pdf_processor,
    '.pgm': pil_processor,
    '.png': pil_processor,
    '.ppm': pil_processor,
    '.pptm': make_zip_processor('Macro-enabled OOXML Presentation'),
    '.pptx': make_zip_processor('OOXML Presentation'),
    '.py': py_processor,
    '.pyc': ignore,
    '.pyo': ignore,
    # TODO: Use an OK message for uncompressed TAR that's clear about
    #       how limited the check is.
    '.ra': make_subproc_processor('RealAudio', ffmpeg_cmd),
    '.rdf': xml_processor,
    '.rm': make_subproc_processor('RealMedia Video', ffmpeg_cmd),
    '.rmvb': make_subproc_processor('RealMedia Video (VBR)', ffmpeg_cmd),
    '.rss': xml_processor,
    '.rv': make_subproc_processor('RealVideo', ffmpeg_cmd),
    '.shn': make_subproc_processor('Shorten Audio', ffmpeg_cmd),
    '.spx': make_subproc_processor('Speex Audio', ffmpeg_cmd),
    '.tar': tar_processor,
    '.tbz2': tar_processor,
    '.tga': pil_processor,
    '.tgz': tar_processor,
    '.tif': pil_processor,
    '.tiff': pil_processor,
    '.tlz': tar_processor,
    '.ts': make_subproc_processor('MPEG Transport Stream', ffmpeg_cmd),
    '.tsa': make_subproc_processor('MPEG Transport Stream Audio', ffmpeg_cmd),
    '.tsv': make_subproc_processor('MPEG Transport Stream Video', ffmpeg_cmd),
    '.txt': make_unverifiable("Plaintext"),
    # NOTE: .war is handled by header detection because it could be a Java WAR
    #       (which is a Zip file) or a Konqueror WAR (which is a TAR file).
    '.txz': tar_processor,
    '.voc': make_subproc_processor('Soundblaster VOC Audio', ffmpeg_cmd),
    '.wav': make_subproc_processor('Microsoft Waveform Audio', ffmpeg_cmd),
    '.webm': make_subproc_processor('WebM Video', ffmpeg_cmd),
    '.webp': pil_processor,
    '.wma': make_subproc_processor('Windows Media Audio', ffmpeg_cmd),
    '.wmv': make_subproc_processor('Windows Media Video', ffmpeg_cmd),
    '.wv': make_subproc_processor('WavPack Audio', ffmpeg_cmd),
    '.xbm': pil_processor,
    '.xlsx': make_zip_processor('OOXML Workbook'),
    '.xlsm': make_zip_processor('Macro-enabled OOXML Workbook'),
    '.xml': xml_processor,
    '.xpi': make_zip_processor('Mozilla XPI'),
    '.xpm': pil_processor,
    '.xz': make_compressed_processor('.xz', lzma),
    '.zip': make_zip_processor('Zip archive'),

    # Formats where redistributable test files cannot be created without paying
    # a license fee:
    '.cbr': make_subproc_processor('Comic Book Archive (RAR)', ['unrar', 't']),
    '.rar': make_subproc_processor('RAR', ['unrar', 't']),
    '.rsn': make_subproc_processor('RSN', ['unrar', 't']),
}

# Callback-based identification with a defined fallback chain
# (Useful for ensuring formats are checked most-likely first)
HEADER_PROCESSORS = (
    # TODO: Figure out how to identify FLAC inside an Ogg container so it can
    #       be checked properly. (ffmpeg doesn't detect my test corruption like
    #       `flac -t` does)
    # TODO: Java and Konqueror .war test files
    (zipfile.is_zipfile, make_zip_processor('unknown Zip-based')),
    (make_header_check(b'SQLite format 3\x00'), sqlite3_processor),

    # TAR check should come before the compressions it might be inside
    (tarfile.is_tarfile, tar_processor),
    (make_header_check(b'\x1f\x8b'), make_compressed_processor('GZip', gzip)),
    (make_header_check(b'BZh'), make_compressed_processor('BZip2', bz2)),
    (make_header_check(b'\xFD7zXZ\x00'),
     make_compressed_processor('.xz', lzma)),

    (make_header_check(b'.snd'),
     make_subproc_processor('Sun Audio (with header)', ffmpeg_cmd)),

    # -- Formats where redistributable test files cannot be created without --
    # -- paying a license fee:                                              --
    (make_header_check(b'\x52\x61\x72\x21\x1A\x07'),
     make_subproc_processor('RAR', ['unrar', 't'])),
    # ------------------------------------------------------------------------

    # TODO: Write a variant of make_header_check that's suited to the
    # non-significant whitespace and comments present in textual formats
    (make_header_check(b'<?xml '), xml_processor),
)

def process_file(path):
    """Check the given path for corruption"""
    log.debug("Processing %r", path)
    fext = os.path.splitext(path)[1].lower()

    if not os.path.exists(path):
        log.error("Path does not exist: %s", path)
    elif os.stat(path).st_size == 0:
        log.error("File is empty: %s", path)
    elif path.endswith('/hts-cache/new.zip'):
        log.info("Skipping expected-to-be-spec-incompliant Zip file: %s",
                 path)
    elif fext in EXT_PROCESSORS:
        EXT_PROCESSORS[fext](path)
    else:
        for check, validator in HEADER_PROCESSORS:
            if check(path):
                validator(path)
                return

        log.error("Unrecognized file type: %s", path)

def walk_path(root):
    """Process the given folder tree"""
    if os.path.isfile(root):
        process_file(root)
    else:
        for path, dirs, files in os.walk(root):
            dirs.sort()
            files.sort()

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
