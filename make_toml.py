#!/usr/bin/env python3
"""Quick hack to convert definitions to TOML"""

from collections import OrderedDict
import toml


def make_rec(handler_name, format_desc, argv=None, use_tmp=False, **kwargs):
    """Argument format adapter"""
    record = OrderedDict({
        'handler': handler_name,
        'description': format_desc,
    })
    if argv:
        record['handler_args'] = argv
    if use_tmp:
        record['use_temporary_dir'] = True

    record.update(kwargs)

    return record

# NOTE: Once I'm not using this anymore and the TOML is
# authoritative, I'll take advantage of the ability to have more
# than one definition for a single extension to do things like
# pointing .cab at both cabextract and unshield.
EXT_PROCESSORS = {
    '.3gp': make_rec('ffmpeg', 'MPEG-4 Part 12 Media (3GPP)'),
    '.3g2': make_rec('ffmpeg', 'MPEG-4 Part 12 Media (3GPP2)'),
    '.7z': make_rec('p7zip', '7-Zip archive', id='7zip',
        header=b'7z\xbc\xaf\x27\x1c'),
    '.aac': make_rec('ffmpeg', 'AAC (ADTS Stream)'),
    '.ape': make_rec('ffmpeg', 'Monkey\'s Audio'),
    '.arj': make_rec('arj', 'ARJ archive'),
    ('.aif', '.aiff'): make_rec('ffmpeg', 'AIFF Audio'),
    '.aifc': make_rec('ffmpeg', 'AIFF Audio (Compressed)'),
    '.asf': make_rec('ffmpeg', 'Microsoft ASF', id='asf',
        header=b'\x30\x26\xB2\x75\x8E\x66\xCF\x11\xA6\xD9\x00\xAA'
        b'\x00\x62\xCE\x6C'),
    '.avi': make_rec('ffmpeg', 'Microsoft AVI Video'),
    '.bin': make_rec('bin', '.bin'),
    ('.bmp', '.dib'): make_rec('pil',
        'Microsoft Device-Independent Bitmap Image'),
    '.bz2': make_rec('bz2', 'BZip2 compressed', id='bzip2',
        header=b'BZh'),
    '.cab': make_rec('cabextract', 'Microsoft CAB'),
    '.cb7': make_rec('p7zip', 'Comic Book Archive (7-Zip)',
        container='7zip'),
    '.cbr': make_rec('unrar', 'Comic Book Archive (RAR)',
        container='rar'),
    '.cbt': make_rec('tar', 'Comic Book Archive (Tar)',
        container='tar'),
    '.cbz': make_rec('zip', 'Comic Book Archive (Zip)',
        container='zip'),
    '.cur': make_rec('pil', 'Microsoft Cursor', multipage=True),
    '.dashtoc': make_rec('json', 'Dash/Zeal Docset Table of Contents'),
    '.dcx': make_rec('pil', 'Multi-page ZSoft PC Paintbrush Image',
        multipage=True),
    '.deb': make_rec('p7zip', 'Debian Package'),
    '.dmg': make_rec('p7zip', 'Apple DMG Disk Image'),
    '.docm': make_rec('zip', 'Macro-enabled OOXML Document',
        container='zip'),
    '.docx': make_rec('zip', 'OOXML Document', container='zip'),
    '.epub': make_rec('zip', 'ePub e-book', container='zip'),
    '.exe': make_rec('innoextract', 'Inno Setup Installer'),
    '.flac': make_rec('flac', 'FLAC Audio'),
    '.f4a': make_rec('ffmpeg', 'FLV Audio'),
    '.f4b': make_rec('ffmpeg', 'FLV Audiobook'),
    '.f4v': make_rec('ffmpeg', 'FLV Video'),
    ('.fli', '.flc'): make_rec('pil', 'Autodesk FLIC Animation'),
    '.flv': make_rec('ffmpeg', 'Flash Video'),
    '.gif': make_rec('pil', 'GIF Image',
        header=[b'GIF87a', b'GIF89a']),
    '.gz': make_rec('gzip', 'GZip compressed', id='gzip',
        header=b'\x1f\x8b'),
    '.hqx': make_rec('binhex', 'BinHex4'),
    '.ico': make_rec('pil', 'Microsoft Icon', multipage=True),
    '.jar': make_rec('zip', 'Java ARchive', container='zip'),
    ('.j2k', '.jp2', '.jpf', '.jpx'): make_rec('pil', 'JPEG 2000 Image'),
    ('.jfi', '.jfif', '.jif', '.jpe', '.jpeg', '.jpg'):
        make_rec('pil', 'JPEG/JFIF Image', header=b'\xff\xd8\xff'),
    '.json': make_rec('json', 'JSON Data'),
    ('.lha', '.lzh'): make_rec('p7zip', 'LHA archive'),
    '.lz': make_rec('lzip', 'Lzip compressed',
        header=b'\x4c\x5a\x49\x50'),
    '.lzma': make_rec('lzma', '.lzma', id='lzma'),
    '.lzx': make_rec('lsar', 'LZX archive'),
    '.m4a': make_rec('ffmpeg', 'MPEG-4 Part 14 Audio'),
    '.m4b': make_rec('ffmpeg', 'MPEG-4 Part 14 Audiobook'),
    '.m4r': make_rec('ffmpeg', 'MPEG-4 Part 14 Ringtone'),
    '.m4v': make_rec('ffmpeg', 'MPEG-4 Part 14 Video'),
    '.mk3d': make_rec('ffmpeg', 'Matroska Video (3D)'),
    '.mka': make_rec('ffmpeg', 'Matroska Audio'),
    '.mkv': make_rec('ffmpeg', 'Matroska Video'),
    '.mov': make_rec('ffmpeg', 'Quicktime Video'),
    '.mp1': make_rec('ffmpeg', 'MPEG Layer 1 Audio'),
    '.mp2': make_rec('ffmpeg', 'MPEG Layer 2 Audio'),
    '.mp3': make_rec('ffmpeg', 'MPEG Layer 3 Audio'),
    '.mp4': make_rec('ffmpeg', 'MPEG-4 Part 14 Video'),
    ('.mp+', '.mpc', '.mpp'): make_rec('ffmpeg', 'Musepack Audio'),
    ('.mpe', '.mpeg', '.mpg'): make_rec('ffmpeg', 'MPEG Video'),
    '.msi': make_rec('p7zip', 'MSI Installer'),
    '.odb': make_rec('zip', 'ODF Database',
        container='zip'),
    '.odc': make_rec('zip', 'ODF Chart',
        container='zip'),
    '.odf': make_rec('zip', 'ODF Formula',
        container='zip'),
    '.odg': make_rec('zip', 'ODF Drawing',
        container='zip'),
    '.odi': make_rec('zip', 'ODF Image',
        container='zip'),
    '.odm': make_rec('zip', 'ODF Master Document',
        container='zip'),
    '.odp': make_rec('zip', 'ODF Presentation',
        container='zip'),
    '.ods': make_rec('zip', 'ODF Spreadsheet',
        container='zip'),
    '.odt': make_rec('zip', 'ODF Text Document',
        container='zip'),
    '.oga': make_rec('ffmpeg', 'Ogg containing audio (.oga)'),
    '.ogg': make_rec('ffmpeg', 'Ogg Vorbis (.ogg)'),
    '.ogm': make_rec('ffmpeg', 'OGM (Ogg, Unofficial)'),
    '.ogv': make_rec('ffmpeg', 'Ogg containing video (.ogv)'),
    '.ogx': make_rec('ffmpeg', 'Ogg (unspecified) (.ogx)'),
    '.opus': make_rec('ffmpeg', 'Opus Audio'),
    '.otc': make_rec('zip', 'ODF Chart Template',
        container='zip'),
    '.otg': make_rec('zip', 'ODF Drawing Template',
        container='zip'),
    '.oth': make_rec('zip', 'ODF Web Page Template',
        container='zip'),
    '.oti': make_rec('zip', 'ODF Image Template',
        container='zip'),
    '.otf': make_rec('zip', 'ODF Formula Template',
        container='zip'),
    '.otp': make_rec('zip', 'ODF Presentation Template',
        container='zip'),
    '.ots': make_rec('zip', 'ODF Spreadsheet Template',
        container='zip'),
    '.ott': make_rec('zip', 'ODF Text Document Template',
        container='zip'),
    '.pbm': make_rec('pil', 'NetPBM Portable Bitmap Image'),
    '.pcx': make_rec('pil', 'ZSoft PC Paintbrush Image'),
    '.pdf': make_rec('pdftotext', 'PDF Document', header=b'%PDF'),
    '.pgm': make_rec('pil', 'NetPBM Portable Graymap Image'),
    '.png': make_rec('pil', 'PNG Image', header=b'\x89PNG\r\n\x1a\n'),
    '.potm': make_rec('zip', 'Macro-enabled OOXML Presentation Template',
        container='zip'),
    '.ppm': make_rec('pil', 'NetPBM Portable Pixmap Image'),
    '.pptm': make_rec('zip', 'Macro-enabled OOXML Presentation',
        container='zip'),
    '.pptx': make_rec('zip', 'OOXML Presentation',
        container='zip'),
    '.ppsx': make_rec('zip', 'OOXML Presentation (Self-Starting)',
        container='zip'),
    '.py': make_rec('py', 'Python Source Code'),
    '.pyc': make_rec('ignore', 'Python Bytecode'),
    '.pyo': make_rec('ignore', 'Python Bytecode (Optimized)'),
    '.ra': make_rec('ffmpeg', 'RealAudio'),
    '.rar': make_rec('unrar', 'RAR', id='rar',
        header=b'\x52\x61\x72\x21\x1A\x07'),
    '.rdf': make_rec('xml', 'RDF Document'),
    '.rm': make_rec('ffmpeg', 'RealMedia Video'),
    '.rmvb': make_rec('ffmpeg', 'RealMedia Video (VBR)'),
    '.rpm': make_rec('rpm', 'RPM Package'),
    '.rsn': make_rec('unrar', 'RSN',
        container='rar'),
    '.rss': make_rec('xml', 'RSS Feed'),
    '.rv': make_rec('ffmpeg', 'RealVideo'),
    '.sea': make_rec('lsar', 'SEA archive'),
    '.shn': make_rec('ffmpeg', 'Shorten Audio'),
    '.sit': make_rec('lsar', 'Stuffit archive'),
    '.spx': make_rec('ffmpeg', 'Speex Audio'),
    '.sqlite3': make_rec('sqlite3', 'SQLite3 Database',
        header=b'SQLite format 3\x00'),
    '.svg': make_rec('xml', 'SVG Image'),
    '.svgz': make_rec('gzip', 'SVG Image (GZip compressed)',
        container='gzip'),
    '.tar': make_rec('tar', 'Tar Archive', id='tar',
        header_offset=257,
        header=[b'ustar\x0000', b'ustar  \0']),
    '.tbz2': make_rec('tar', 'Tar Archive (BZip2 compressed)',
        container='bzip2'),
    '.tga': make_rec('pil', 'Truevision TGA Image'),
    '.tgz': make_rec('tar', 'Tar Archive (GZip compressed)',
        container='gzip'),
    ('.tif', '.tiff'): make_rec('pil', 'TIFF Image'),
    '.tlz': make_rec('tar', 'Tar Archive (.lzma compressed)',
        container='lzma'),
    '.ts': make_rec('ffmpeg', 'MPEG Transport Stream'),
    '.tsa': make_rec('ffmpeg', 'MPEG Transport Stream Audio'),
    '.tsv': make_rec('ffmpeg', 'MPEG Transport Stream Video'),
    '.txt': make_rec('txt', 'Plaintext'),
    '.txz': make_rec('tar', 'Tar Archive (.xz compressed)',
        container='xz'),
    ('.uu', '.uue'): make_rec('uuencode', 'UUEncoded'),
    '.voc': make_rec('ffmpeg', 'Soundblaster VOC Audio'),
    '.wav': make_rec('ffmpeg', 'Microsoft Waveform Audio'),
    '.webm': make_rec('ffmpeg', 'WebM Video'),
    '.webp': make_rec('pil', 'WebP Image'),
    '.wma': make_rec('ffmpeg', 'Windows Media Audio',
        container='asf'),
    '.wmv': make_rec('ffmpeg', 'Windows Media Video',
        container='asf'),
    '.wv': make_rec('ffmpeg', 'WavPack Audio'),
    '.xbm': make_rec('pil', 'X BitMap Image'),
    '.xlsx': make_rec('zip', 'OOXML Workbook',
        container='zip'),
    '.xlsm': make_rec('zip', 'Macro-enabled OOXML Workbook',
        container='zip'),
    '.xml': make_rec('xml', 'XML Data',
        header=b'<?xml '),
    '.xpi': make_rec('zip', 'Mozilla XPI',
        container='zip'),
    '.xpm': make_rec('pil', 'X PixMap Image'),
    '.xz': make_rec('lzma', '.xz compressed', id='xz',
        header=b'\xFD7zXZ\x00'),
    '.zip': make_rec('zip', 'Zip archive', id='zip',
        header=[b'PK\x03\x04', b'PK\x05\x06', b'PK\x07\x08']),
}


def exts_key(x):
    """Key function for sorting EXT_PROCESSORS by extension"""
    if isinstance(x, tuple):
        return x[0]
    else:
        return x

extensions = []
for ext in sorted(EXT_PROCESSORS, key=exts_key):
    record = EXT_PROCESSORS[ext]
    if isinstance(ext, tuple):
        ext = list(ext)
    record['extension'] = ext
    extensions.append(record)

extensions.append(OrderedDict({
    'description': "Sun Audio (with header)",
    'header': b'.snd',
    'handler': 'ffmpeg',
}))

BUILTIN_PROCESSORS = [
    'bin', 'binhex', 'bz2', 'gzip', 'ignore', 'json', 'lzma', 'pil',
    'py', 'sqlite3', 'tar', 'txt', 'unverifiable', 'uuencode', 'xml', 'zip']
PROCESSORS = {
    'arj': {
        'argv': ['arj', 't'],
    },
    'cabextract': {
        'argv': ['cabextract', '-t'],
    },
    'ffmpeg': {
        'argv': ['ffmpeg', '-f', 'null', '-', '-i'],
    },
    'flac': {
        'argv': ['flac', '-t'],
    },
    'innoextract': {
        'argv': ['innoextract', '-t', '-g'],
    },
    'lzip': {
        'argv': ['lzip', '-t'],
    },
    'p7zip': {
        'argv': ['7z', 't']
    },
    'pdftotext': {
        'argv': ['pdftotext', '{path}', '{devnull}'],
        'fail_if_stderr': 'Error',

    },
    'rpm': {
        'argv': ['rpm', '--checksig'],
    },
    'lsar': {
        'argv': ['lsar', '-t'],
    },
    'unrar': {
        'argv': ['unrar', 't'],
    },
}

# Generate the TOML and remove the trailing commas
output = OrderedDict([
        ('filetype', extensions),
        ('handler', PROCESSORS),
        ('override', [
            {
                'path': '*/hts-cache/new.zip',
                'type': 'ignore',
                'message': 'Skipping intentionally broken HTTrack Zip file',
            },
            {
                'path': '*/.git',
                'recurse': False,
            }

        ])
])
out_str = toml.dumps(output).replace(',]', ' ]')

# Verify that the manual tweaks didn't break round-tripping
comparison = toml.loads(out_str)
for val in comparison['filetype']:
    if 'header' in val:
        header = val['header']
        if isinstance(header[0], list):
            val['header'] = [bytes(x) for x in header]
        else:
            val['header'] = bytes(header)

assert output == comparison  # nosec

with open('verifiers.toml', 'w') as fobj:
    fobj.write(out_str)

for record in extensions:
    if record['handler'] not in (BUILTIN_PROCESSORS +
            list(PROCESSORS.keys())):
        print("HANDLER FOR {} NOT REGISTERED: {} ".format(
            record['extension'], record['handler']))
