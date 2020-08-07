#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""A simple tool to recursively walk a set of paths and report files where
corruption was detected.
"""

from __future__ import (absolute_import, division, print_function,
                        with_statement, unicode_literals)

__author__ = "Stephan Sokolow (deitarion/SSokolow)"
__appname__ = "Simple recursive detector for corrupted files"
__version__ = "0.0pre0"
__license__ = "GNU GPL 3.0 or later"

import logging, os, re, tempfile
import ast, binhex, bz2, codecs, gzip, json, lzma, sqlite3, tarfile, zipfile
import subprocess  # nosec
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


def bin_processor(path):
    """Conditionally ignore InnoSetup BIN files"""
    with open(path, 'rb') as fobj:
        is_innosetup = (fobj.read(7) == b"idska32") or rar_header_check(path)

    if not is_innosetup:
        log.error("Unrecognized file type: %s", path)
        return

    base_name = re.match(r'(.*)-\d+\.bin', path, re.I)
    if not base_name:
        log.error("Filename is not a valid InnoSetup BIN pattern: %s", path)
        return

    exe_name = base_name.group(1) + '.exe'
    if not os.path.exists(exe_name):  # TODO: Support case-insensitivity
        log.error("No corresponding InnoSetup EXE file: %s", path)
        return

    log.info("Skipping .bin file for %s: %s", os.path.basename(exe_name), path)


def binhex_processor(path):
    """Verify the given path is a valid, uncorrupted binhex4 file."""
    try:
        with tempfile.TemporaryDirectory(prefix='verify_files-') as tmpdir:
            with open(path, 'rb') as fobj:
                binhex.hexbin(fobj, os.path.join(tmpdir, "out"))
            return True
    except Exception as err:  # pylint: disable=broad-except
        log.error("BinHex file verification failed: %s", path)
        log.debug("...because: %s: %s", err.__class__.__name__, err)
        return False


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


def txt_processor(path):
    """Do a basic check for suspicious null bytes in a plaintext file"""
    try:
        with open(path, mode='rb') as fobj:
            raw = fobj.read()

        boms = [
            (codecs.BOM_UTF32_LE, 'utf-32-le'),
            (codecs.BOM_UTF32_BE, 'utf-32-be'),
            (codecs.BOM_UTF16_LE, 'utf-16-le'),
            (codecs.BOM_UTF16_BE, 'utf-16-be'),
            (codecs.BOM_UTF8, 'utf-8'),
        ]

        for bom, encoding in boms:
            if not raw.startswith(bom):
                continue

            try:
                text = raw.decode(encoding)
                if '\0' in text:
                    raise ValueError("A null byte in a {}-encoded .txt file "
                        "is likely to be corruption".format(encoding.upper()))
                break
            except UnicodeDecodeError:
                raise ValueError("File began with BOM for {} but decoding "
                    "failed".format(encoding))

        # BOM-less UTF-32-compatible check for null bytes which will also catch
        # two sequential nulls in UTF-16 as long as they're aligned properly
        if all(b'\0' in raw[x::4] for x in range(4)):
            raise ValueError("A null byte in a .txt file is likely corruption")
    except Exception as err:  # pylint: disable=broad-except
        log.error("Plaintext verification failed: %s", path)
        log.debug("...because: %s: %s", err.__class__.__name__, err)
    else:
        log.warning("Plaintext files have no checksum: %s", path)


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


def make_subproc_processor(fmt_name, argv_prefix, argv_suffix=None,
                           use_tmp=False):
    """Closure factory for formats verifiable by subprocess exit code"""
    argv_suffix = argv_suffix or []

    def process(path):
        """Verify the given path"""
        if not which(argv_prefix[0]):
            log.warning("Must install %s to test %s files",
                        argv_prefix[0], fmt_name)
            return

        argv = argv_prefix + [os.path.abspath(path)] + argv_suffix
        with open(os.devnull, 'w') as nul:
            try:
                if use_tmp:
                    with tempfile.TemporaryDirectory(prefix='verify-') as tdir:
                        subprocess.check_call(  # nosec
                            argv, cwd=tdir, stdout=nul, stderr=nul)
                else:
                    subprocess.check_call(  # nosec
                        argv, stdout=nul, stderr=nul)
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
    """Run SQLite 3.x's integrity_check and foreign_key_check pragmas"""
    try:
        db = sqlite3.connect(path)
        db.execute('PRAGMA integrity_check')  # Raises exception

        broken_foreigns = db.execute('PRAGMA foreign_key_check').fetchone()
        if broken_foreigns:
            raise Exception("Broken foreign key references detected: {!r}"
                            .format(broken_foreigns))
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

rar_header_check = make_header_check(b'\x52\x61\x72\x21\x1A\x07')

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
    '.bin': bin_processor,
    '.bmp': pil_processor,
    '.bz2': make_compressed_processor('BZip2', bz2),
    # TODO: Also support InstallShield CABs and use a custom message to
    # acknowledge that it might also be just an unsupported variant
    '.cab': make_subproc_processor('Microsoft CAB', ['cabextract', '-t']),
    '.cb7': make_subproc_processor('Comic Book Archive (7-Zip)', ['7z', 't']),
    '.cbz': make_zip_processor('Comic Book Archive (Zip)'),
    '.cbt': tar_processor,
    '.cur': pil_multi_processor,
    '.dashtoc': json_processor,
    '.dcx': pil_multi_processor,
    '.deb': make_subproc_processor('.deb', ['7z', 't']),  # TODO: Test file
    '.dmg': make_subproc_processor('.dmg', ['7z', 't']),  # TODO: Test file
    '.dib': pil_processor,
    '.docm': make_zip_processor('Macro-enabled OOXML Document'),
    '.docx': make_zip_processor('OOXML Document'),
    '.epub': make_zip_processor('ePub e-book'),
    # TODO: Support other kinds of EXEs too
    '.exe': make_subproc_processor('Inno Setup Installer',
                                   ['innoextract', '-t', '-g']),
    '.flac': make_subproc_processor('FLAC', ['flac', '-t']),
    '.f4a': make_subproc_processor('FLV Audio', ffmpeg_cmd),
    '.f4b': make_subproc_processor('FLV Audiobook', ffmpeg_cmd),
    '.f4v': make_subproc_processor('FLV Video', ffmpeg_cmd),
    '.fli': pil_processor,
    '.flc': pil_processor,
    '.flv': make_subproc_processor('Flash Video', ffmpeg_cmd),
    '.gif': pil_processor,
    '.gz': make_compressed_processor('GZip', gzip),
    '.hqx': binhex_processor,
    '.ico': pil_multi_processor,
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
    '.lha': make_subproc_processor('LHA archive', ['7z', 't']),
    '.lz': make_subproc_processor('Lzip', ['lzip', '-t']),
    '.lzh': make_subproc_processor('LHA archive', ['7z', 't']),
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
    '.msi': make_subproc_processor('MSI Installer', ['7z', 't']),
    '.odb': make_zip_processor('ODF Database'),
    '.odc': make_zip_processor('ODF Chart'),
    '.odf': make_zip_processor('ODF Formula'),
    '.odg': make_zip_processor('ODF Drawing'),
    '.odi': make_zip_processor('ODF Image'),
    '.odm': make_zip_processor('ODF Master Document'),
    '.odp': make_zip_processor('ODF Presentation'),
    '.ods': make_zip_processor('ODF Spreadsheet'),
    '.odt': make_zip_processor('ODF Text Document'),
    '.oga': make_subproc_processor('Ogg containing audio (.oga)', ffmpeg_cmd),
    '.ogg': make_subproc_processor('Ogg Vorbis (.ogg)', ffmpeg_cmd),
    '.ogm': make_subproc_processor('OGM (Ogg, Unofficial)', ffmpeg_cmd),
    '.ogv': make_subproc_processor('Ogg containing video (.ogv)', ffmpeg_cmd),
    '.ogx': make_subproc_processor('Ogg (unspecified) (.ogx)', ffmpeg_cmd),
    '.opus': make_subproc_processor('Opus Audio', ffmpeg_cmd),
    '.otc': make_zip_processor('ODF Chart Template'),
    '.otg': make_zip_processor('ODF Drawing Template'),
    '.oth': make_zip_processor('ODF Web Page Template'),
    '.oti': make_zip_processor('ODF Image Template'),
    '.otf': make_zip_processor('ODF Formula Template'),
    '.otp': make_zip_processor('ODF Presentation Template'),
    '.ots': make_zip_processor('ODF Spreadsheet Template'),
    '.ott': make_zip_processor('ODF Text Document Template'),
    '.pbm': pil_processor,
    '.pcx': pil_processor,
    '.pdf': pdf_processor,
    '.pgm': pil_processor,
    '.png': pil_processor,
    '.potm': make_zip_processor('Macro-enabled OOXML Presentation Template'),
    '.ppm': pil_processor,
    '.pptm': make_zip_processor('Macro-enabled OOXML Presentation'),
    '.pptx': make_zip_processor('OOXML Presentation'),
    '.ppsx': make_zip_processor('OOXML Presentation (Self-Starting)'),
    '.py': py_processor,
    '.pyc': ignore,
    '.pyo': ignore,
    # TODO: Use an OK message for uncompressed TAR that's clear about
    #       how limited the check is.
    '.ra': make_subproc_processor('RealAudio', ffmpeg_cmd),
    '.rdf': xml_processor,
    '.rm': make_subproc_processor('RealMedia Video', ffmpeg_cmd),
    '.rmvb': make_subproc_processor('RealMedia Video (VBR)', ffmpeg_cmd),
    '.rpm': make_subproc_processor('RPM Package', ['rpm', '--checksig']),
    '.rss': xml_processor,
    '.rv': make_subproc_processor('RealVideo', ffmpeg_cmd),
    '.shn': make_subproc_processor('Shorten Audio', ffmpeg_cmd),
    '.spx': make_subproc_processor('Speex Audio', ffmpeg_cmd),
    '.svg': xml_processor,  # TODO: Validate more thoroughly?
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
    '.txt': txt_processor,
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

    # Formats I may need to use my Amiga Forever license to generate test files
    # for:
    '.lzx': make_subproc_processor('LZX archive', ['unar'], use_tmp=True),
    # Formats where redistributable test files cannot be created without paying
    # a license fee:
    '.cbr': make_subproc_processor('Comic Book Archive (RAR)', ['unrar', 't']),
    '.rar': make_subproc_processor('RAR', ['unrar', 't']),
    '.rsn': make_subproc_processor('RSN', ['unrar', 't']),
    # Formats which may be in the same boat:
    '.sit': make_subproc_processor('Stuffit archive', ['unar'], use_tmp=True),
    '.sea': make_subproc_processor('SEA archive', ['unar'], use_tmp=True),
}

# Callback-based identification with a defined fallback chain
# (Useful for ensuring formats are checked most-likely first)
HEADER_PROCESSORS = (
    # TODO: Figure out how to identify FLAC inside an Ogg container so it can
    #       be checked properly. (ffmpeg doesn't detect my test corruption like
    #       `flac -t` does)
    # TODO: Java and Konqueror .war test files
    # TODO: Don't let a match for a header-based format block a match for a
    #       trailer-based format. (eg. a PNG or EXE with a Zip concatenated)

    # Zip files should come first since it's the most popular basis for
    # exchange formats which has a magic number for *reliabl* header-detection.
    # (Unlike, for example, JSON)
    (zipfile.is_zipfile, make_zip_processor('unknown Zip-based')),
    (make_header_check(b'SQLite format 3\x00'), sqlite3_processor),

    # TAR check should come before the compressions it might be inside and
    # GZip should come as early as possible after Zip for similar reasons.
    (tarfile.is_tarfile, tar_processor),
    (make_header_check(b'\x1f\x8b'), make_compressed_processor('GZip', gzip)),
    (make_header_check(b'BZh'), make_compressed_processor('BZip2', bz2)),
    (make_header_check(b'\xFD7zXZ\x00'),
     make_compressed_processor('.xz', lzma)),

    # -- Formats where redistributable test files cannot be created without --
    # -- paying a license fee:                                              --
    (rar_header_check, make_subproc_processor('RAR', ['unrar', 't'])),
    # ------------------------------------------------------------------------

    # TODO: Write a variant of make_header_check that's suited to the
    # non-significant whitespace and comments present in textual formats
    (make_header_check(b'<?xml '), xml_processor),
    (make_header_check(b'%PDF'), pdf_processor),

    # TODO: Match b"GIF87a" or b"GIF89a" and nothing else once the checks have
    # been rewritten to only read the file once.
    (make_header_check(b'GIF8'), pil_processor),
    (make_header_check(b'\xff\xd8\xff'), pil_processor),  # JPEG
    (make_header_check(b'\x89PNG\r\n\x1a\n'), pil_processor),

    (make_header_check(b'7z\xbc\xaf\x27\x1c'),
     make_subproc_processor('7-Zip archive', ['7z', 't'])),
    (make_header_check(b'\x4c\x5a\x49\x50'),
     make_subproc_processor('Lzip', ['lzip', '-t'])),

    (make_header_check(b'.snd'),
     make_subproc_processor('Sun Audio (with header)', ffmpeg_cmd)),
)


def process_file(path, list_unrecognized=False):
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
        if not list_unrecognized:
            EXT_PROCESSORS[fext](path)
    else:
        for check, validator in HEADER_PROCESSORS:
            if check(path):
                if not list_unrecognized:
                    validator(path)
                return

        log.error("Unrecognized file type: %s", path)


def walk_path(root, list_unrecognized=False):
    """Process the given folder tree"""
    if os.path.isfile(root):
        process_file(root, list_unrecognized)
    elif os.path.isdir(root):
        for path, dirs, files in os.walk(root):
            dirs.sort()
            files.sort()

            if '.git' in dirs:
                dirs.remove('.git')
                # TODO: `git fsck`

            for fname in files:
                fpath = os.path.join(path, fname)
                process_file(fpath, list_unrecognized)
    else:
        log.error("Bad path: %s", root)


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
    parser.add_argument('--list-unrecognized', action="store_true",
        default=False,
        help="Just quickly identify files that have no checker registered")
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
        walk_path(path, args.list_unrecognized)

if __name__ == '__main__':
    main()

# vim: set sw=4 sts=4 expandtab :
