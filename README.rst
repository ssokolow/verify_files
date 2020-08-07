===============
verify_files.py
===============

A simple helper script to recursively check a tree of files for corruption to
the greatest degree I can figure out how to accomplish.

**NOTE:** This script is usable, but I consider it to be in the early stages.
That means that:

1. I have yet to write a test suite to verify that the checks are as thorough
   as I think they are.
2. It currently doesn't aim to do any sort of format validation beyond the
   kinds which will catch corruption of a file that began valid. (eg. If you've
   got invalid or corrupted files that made their way into a Zip-format
   container (which includes formats like ePub and ODF), but the container
   reports that all files pass their CRC checks, no warnings or errors will be
   raised. I'll eventually implement an option to unpack such archives and test
   each file within them.)

It's also *de facto* targeted at Linux users in that, while I followed my
usual practises of writing portable code, installing and adding a whole bunch
of little helper utilities to your ``PATH`` is not convenient outside the Linux
package manager ecosystem.

I may rewrite this in Rust later, so that I can statically link as many of my
dependencies as possible into a single easy-to-install binary.

If you're looking for something more mature but supporting a narrower selection
of formats, I'd suggest looking at JHOVE_.

.. _JHOVE: http://jhove.openpreservation.org/

Dependencies
============

- `Python`_ 3.3+

Optional:
---------

- cabextract_  (for checking Microsoft Cabinet archives)
- defusedxml_  (for checking XML files, no unsafe fallback is provided)
- ffmpeg_      (for checking non-FLAC media files)
- flac_        (for checking FLAC files)
- innoextract_ (for checking InnoSetup installers)
- lzip_        (for checking lzip-compressed files)
- p7zip_       (for checking 7-Zip, .cb7, .deb, .dmg, LHA, and MSI files)
- pdftotext_   (for checking PDFs)
- Pillow_      (for checking images)
- RPM_         (for checking RPM packages)
- unar_        (for checking LZX and Stuffit archives)
- unrar_       (for checking RAR/CBR/RSN files)

.. _cabextract: https://www.cabextract.org.uk/
.. _defusedxml: https://pypi.org/project/defusedxml/
.. _flac: https://xiph.org/flac/
.. _innoextract: https://constexpr.org/innoextract/
.. _lzip: http://lzip.nongnu.org/
.. _p7zip: http://p7zip.sourceforge.net/
.. _pdftotext: https://en.wikipedia.org/wiki/Pdftotext
.. _Pillow: https://python-pillow.org/
.. _Python: https://www.python.org/
.. _RPM: http://rpm.org/
.. _unar: https://packages.debian.org/stable/unar
.. _unrar: https://www.rarlab.com/rar_add.htm


Format Support
==============

With the appropriate dependencies installed, this tool can do at least some
form of basic corruption check on the following formats.

In addition, the following special-case rules will be applied:

* An empty file will always trigger an error message.
* The ``hts-cache/new.zip`` files in HTTrack_ archives will be skipped as they
  do not properly comply with the Zip specification and trigger spurious
  complaints from Info-ZIP.
* Files which

Header/Trailer-based Identification
-----------------------------------

Any files which haven't been assigned a more specific check based on their
extensions will be identified based on their contents and checked as follows:

**BZip2:**
    Any file which begins with a BZip2 header will be checked for corruption
    using the format's internal CRC.
**GZip:**
    Any file which begins with a GZip header will be checked for corruption
    using the format's internal CRC.
**SQLite 3.x:**
    Any file which begins with an SQLite 3.x header will be checked for
    corruption using SQLite's ``PRAGMA integrity_check``.

    Note that SQLite 3.x does not checksum data stored within it, so this can
    only detect corruption which damages the database's structure. It will not
    catch corruption which exists entirely within the stored data.
**RAR:**
    Any file which begins with a RAR header (pre- or post-5.x) will be fed to
    the ``unrar t`` command to be checked for corruption.
**Sun Audio (.au, .snd):**
    Because ``.snd`` had such an overloaded meaning in the DOS era and ffmpeg
    seems to have trouble making sense of the ``.au`` files within Audacity
    project folders, only the headered variant of Sun Microsystems audio files
    are supported and they are detected by their headers, rather than their
    extensions.
**TAR:**
    Any file for which ``tarfile.is_tarfile`` returns ``True`` will have a
    ``--list`` operation run on it as a basic structural check.

    Note that the TAR format does not provide any sort of integrity check of
    its own, so corruption within files will only be detected if the archive
    has been compressed within another format which does provide this function,
    such as GZip or BZip2.
**XML:**
    Any file which begins with ``<?xml`` followed by a space will be checked
    to see if it can be parsed.

    In the future, this header check will be made more lenient to accomodate
    variations in whitespace without devolving into a generic substring match
    for ``<?xml`` inside a file which might actually be something else.
**XZ:**
    Any file which begins with an XZ header will be checked for corruption
    using the format's internal integrity checks.
**Zip:**
    Any file for which ``zipfile.is_zipfile`` returns ``True`` will have the
    files within checked for corruption using the format's internal CRCs (and
    you'd be surprised how many formats use Zip as a container).

With the exception of Sun Audio, RAR, and XML, all of these formats are
supported via Python's standard library and require no additional utilities to
be installed.

Extension-based Identification
------------------------------

**Archives:**
    ``.7z``, ``.arj``, ``.dmg``, ``.hqx``, ``.lha``, ``.lzh``, ``.lzx``,
    ``.msi``, ``.rar``, ``.sit``, ``.sea``, ``.tar``, ``.tbz2``, ``.tgz``,
    ``.txz``, and ``.zip`` files will be fed to the appropriate archive tool's
    test function. This will generally perform a CRC check on the compressed
    data, but some files may also contain more robust integrity information.

    At present, unpacking archives to check the files inside for corruption
    introduced before the archive was created is not supported, but is planned.

    Also, be aware that uncompressed TAR archives do not contain CRCs or other
    hashes of the stored data and, as such, cannot be verified beyond
    running ``--list`` on them as the most primitive form of integrity check.
**Audio & Video (General):**
    Files with the following extensions will be fed to ffmpeg_ for decoding.

    * 3GPP_: ``.3gp``, ``.3g2``
    * AAC_ (ADTS Stream): ``.aac``
    * Apple QuickTime_: ``.mov``
    * AIFF_: ``.aif``, ``.aifc``, ``.aiff``
    * `Creative Voice`_ Audio: ``.voc``
    * `Monkey's Audio`_: , ``.ape``
    * `Microsoft ASF`_: ``.asf``, ``.wma``, ``.wmv``
    * `Microsoft AVI`_: ``.avi``
    * `Microsoft Waveform Audio`_: ``.wav``
    * `Flash Video`_: ``.f4a``, ``.f4b``, ``.f4v``, ``.flv``
    * Matroska_ Containers: ``.mk3d``, ``.mka``, ``.mkv``
    * `MPEG-1 Video`_: ``.mpe``, ``.mpeg``, ``.mpg``
    * `MPEG-4 Part 14`_ Containers: ``.m4a``, ``.m4b``, ``.m4r``, ``.m4v``, ``.mp4``
    * `MPEG Audio Layer I`_: ``.mp1``
    * `MPEG Audio Layer II`_: ``.mp2``
    * `MPEG Audio Layer III`_: ``.mp3``
    * `MPEG Transport Stream`_: ``.ts``, ``.tsa``, ``.tsv``
    * Musepack_ Audio: ``.mp+``, ``.mpc``, ``.mpp``
    * Ogg_ Containers: ``.oga``, ``.ogg``, ``.ogm``, ``.ogv``, ``.ogx``
    * RealMedia_ and related formats: ``.ra``, ``.rm``, ``.rmvb``, ``.rv``
    * WavPack_ Audio:  ``.wv``
    * WebM_: ``.webm``

    **CAUTION:** This should not be relied on to make use of all available
    integrity-checking measures.

    For example, ffmpeg will not detect the induced corruption in the FLAC test
    file which is caught by ``flac -t`` validating the embedded MD5 checksum,
    which means that Ogg FLAC files are not currently being checked as
    thoroughly as possible.

    Resolving this shortcoming will require augmenting this tool to inspect Ogg
    containers to identify the formats stored within.
**Audio & Video (.flac Files):**
    The FLAC encoder calculates an MD5 hash of the source audio data during the
    compression process and then stores it in the FLAC file's header.

    This tool will use ``flac -t`` to verify that the audio in files with
    ``.flac`` extensions can still successfully be decoded into audio that is
    bit-for-bit identical to the input file.

    However, to the best of my knowledge, there is no provision for detecting
    corruption in metadata tags and it does not currently detect FLAC content
    within Ogg containers as being testable with ``flac -t``.
**Chiptune Bundles:**
    ``.rsn`` files are just sets of SPC-format chiptunes stored in RAR archives
    and will be checked as archives.
**Comic Book Archives:**
    ``.cb7``, ``.cbz``, ``.cbr``, ``.cbt`` are just renamed 7-Zip, Zip, RAR,
    and TAR archives, respectively, and will be checked as archives.
**Compressed Single Files:**
    ``.bz2``, ``.gz``, ``.lz``, ``.lzma``, and ``.xz`` files will be fed to the
    appropriate decompressor's test function to verify their internal CRCs.
**Debian Packages:**
    ``.deb`` files will be fed to p7zip_'s extraction test function.

    However, ``.deb`` files contain both a control archive and a data archive
    and I suspect this may only be verifying the data archve. As such, I plan
    to redesign this check after building a test suite if it confirms my
    suspicions.
**Images:**
    Files with the following extensions will be loaded using Pillow_ to verify
    that the structure is intact enough to satisfy Pillow's image loader. This
    may or may not involve running proper integrity checks, depending on the
    format.

    * Autodesk FLIC_: ``.flc``, ``.fli``
    * `GIF`_: ``.gif``
    * `JPEG`_: ``jfi``, ``.jfi``, ``.jfif``, ``.jpe``, ``.jpeg``, ``.jpg``
    * `JPEG 2000`_: ``.j2k``, ``.jp2``, ``.jpf``, ``.jpx``
    * Microsoft `Device-Independent Bitmap`_: ``.dib``, ``.bmp``
    * Microsoft Icons and Cursors: ``.cur``, ``.ico``
    * `Netpbm formats`_: ``.pbm``, ``.pgm``, ``.ppm``, ``.pnm``
    * `PC Paintbrush`_: ``.pcx``, ``.dcx``
    * `PNG`_: ``.png``
    * `TIFF`_: ``.tif``, ``.tiff``
    * `Truevision TGA`_: ``.tga``
    * `WebP`_: ``.webp``
    * `X BitMap`_: ``.xbm``
    * `X PixMap`_: ``.xpm``

    **NOTE:** Currently only one image is loaded from the following multi-image
    formats, limiting the utility of this tool for checking them for
    corruption:

    * ``.cur`` (largest available only)
    * ``.dcx`` (first image only)
    * ``.ico`` (largest available only)

    In the future, this check may be extended to identify common artifacts of
    corruption that don't interfere with image loading, such as the distinctive
    bars of nonsense color at the bottom of certain types of corrupted JPEGs.

**InnoSetup EXE Files:**
    ``.exe`` files will be fed to ``innoextract -t -g`` on the assumption that
    they are InnoSetup installers. This will also verify any accompanying
    ``.bin`` files, whether they're InnoSetup's native split-file format or the
    RAR files that GOG.com briefly used.
**JSON Data:**
    ``.json`` and ``.dashtoc`` files will be loaded using the JSON parser from
    the Python standard library as a basic well-formedness check.

    Due to the format's lack of a `magic number`_, JSON files with unfamiliar
    extensions will **not** be recognized.
**Microsoft Cabinet Files:**
    ``.cab`` files will be fed to ``cabextract -t`` to check their internal
    checksums.
**PDF Documents:**
    The PDF format makes no provisions for internal checksumming. However, as
    with any structured markup, some degree of corruption detection *is*
    possible.

    Files with a ``.pdf`` extension will be fed into ``pdftotext`` as it has
    been demonstrated to report failure when it recognizes that the markup
    is not well-formed.
**Plaintext Files:**
    Files with a ``.txt`` extension have no means of checking for corruption
    but will be read from disk in full in order to:

    1. Catch any corruption which is detectable at the level of the filesystem
       or disk firmware.
    2. Perform some heuristic checks for null codepoints, which should not
       occur in ``.txt`` files (text editors like ``NOTEPAD.exe`` treat them as
       the end of the file) but could be inserted by a recovery operation that
       represents an unreadable filesystem/media block as a span of nulls.
**RPM Packages:**
    Files with a ``.rpm`` extension will be fed to RPM's ``--checksig`` mode.

    (Note that not all of the metadata in an RPM file is covered by the
    signatures in question.)
**XML, RDF, RSS, and SVG Files:**
    Files with an ``.rdf``, ``.rss``, ``.svg``, or ``.xml`` extension will be
    parsed to verify that their markup is well-formed.

.. _3GPP: https://en.wikipedia.org/wiki/3GP_and_3G2
.. _AAC: https://en.wikipedia.org/wiki/Advanced_Audio_Coding
.. _AIFF: https://en.wikipedia.org/wiki/AIFF
.. _Creative Voice: https://en.wikipedia.org/wiki/Creative_Voice_file
.. _Device-Independent Bitmap: https://en.wikipedia.org/wiki/BMP_file_format
.. _ffmpeg: https://ffmpeg.org/
.. _FLIC: https://en.wikipedia.org/wiki/FLIC_(file_format)
.. _Flash Video: https://en.wikipedia.org/wiki/Flash_Video
.. _GIF: https://en.wikipedia.org/wiki/GIF
.. _HTTrack: https://www.httrack.com/
.. _JPEG: https://en.wikipedia.org/wiki/JPEG
.. _JPEG 2000: https://en.wikipedia.org/wiki/JPEG_2000
.. _magic number: https://en.wikipedia.org/wiki/List_of_file_signatures
.. _Matroska: https://en.wikipedia.org/wiki/Matroska
.. _Microsoft ASF: https://en.wikipedia.org/wiki/Advanced_Systems_Format
.. _Microsoft AVI: https://en.wikipedia.org/wiki/Audio_Video_Interleave
.. _Microsoft Waveform Audio: https://en.wikipedia.org/wiki/WAV
.. _Monkey's Audio: https://en.wikipedia.org/wiki/Monkey's_Audio
.. _MPEG-1 Video: https://en.wikipedia.org/wiki/MPEG-1
.. _MPEG-4 Part 14: https://en.wikipedia.org/wiki/MPEG-4_Part_14
.. _MPEG Audio Layer I: https://en.wikipedia.org/wiki/MPEG-1_Audio_Layer_I
.. _MPEG Audio Layer II: https://en.wikipedia.org/wiki/MPEG-1_Audio_Layer_II
.. _MPEG Audio Layer III: https://en.wikipedia.org/wiki/MP3
.. _MPEG Transport Stream: https://en.wikipedia.org/wiki/MPEG_transport_stream
.. _Musepack: https://en.wikipedia.org/wiki/Musepack
.. _Netpbm formats: https://en.wikipedia.org/wiki/Netpbm_format
.. _Ogg: https://en.wikipedia.org/wiki/Ogg
.. _PC Paintbrush: https://en.wikipedia.org/wiki/PCX
.. _PNG: https://en.wikipedia.org/wiki/Portable_Network_Graphics
.. _QuickTime: https://en.wikipedia.org/wiki/QuickTime_File_Format
.. _RealMedia: https://en.wikipedia.org/wiki/RealMedia
.. _TIFF: https://en.wikipedia.org/wiki/TIFF
.. _Truevision TGA: https://en.wikipedia.org/wiki/Truevision_TGA
.. _WavPack: https://en.wikipedia.org/wiki/WavPack
.. _WebM: https://en.wikipedia.org/wiki/WebM
.. _WebP: https://en.wikipedia.org/wiki/WebP
.. _X BitMap: https://en.wikipedia.org/wiki/X_BitMap
.. _X PixMap: https://en.wikipedia.org/wiki/X_PixMap

Roadmap
=======

While I haven't decided on a solid order yet, here are my plans for future
improvements:

* Add a command-line option to exclude files/folders when recursing
* Write a test suite (delayed pending the creation of a full set of corrupted
  test files that I can legally redistribute because I need to investigate each
  file format so I know which kinds of corrupt bytes to introduce and where
  in order to produce the most useful tests.)
* Once I have proper fallback chains, support using p7zip to check every
  format that it supports, rather than just 7-zip. (This will also have the
  benefit of not raising false positives on Zip files using features not
  supported by Python's ``zipfile`` module.)
* Add header checks for as many supported formats as possible and then use them
  as an additional means of verifying correctness in addition to their current
  role as a fallback means of finding a checker for files with unrecognized
  extensions.

  * I'll want a more optimized approach to reading headers which minimizes the
    amount of wasted syscalling and disk reading. (Something like reading the
    first 4K chunk of the file, then just passing the resulting bytestring to
    each header inspector in turn.)

* See if I can reuse any code from diffoscope_


Ideas for Further Checks
------------------------

Sorted by a rough approximation of the order I expect to tackle them.

**Plaintext Files:**
    Maybe I can also check for use of ``FF`` bytes, since that's the other
    common fill byte for failed reads.
**.ini, .rc, .desktop, and .conf Files:**
    See what I can do to check these for well-formedness using the parsers in
    Python's standard library.
**Shell Scripts:**
    Can bash do a basic syntax check on untrusted scripts safely?
**git repositories:**
    Verify repositories using `git fsck` and figure out how
    to check the working tree against the repository.
**.exe and .dll files:**
    Verify both the executable part of a ``.exe`` and potential
    appended archives

    * It `doesn't <https://www.mono-project.com/docs/faq/security/>`_ have the
      certificates installed by default, but Mono_ has an implementation of the
      ``chktrust`` tool for verifying Authenticode signatures.
    * I'll want ``innoextract -t`` to be an "archive unpacker" that *only* gets
      used in the fallback chain for self-extractors.
    * I'll want check ordering to be flexible enough to defer ``.bin`` until
      after ``.exe`` of the same prefix so I can catch ``.bin`` files that
      match a ``.exe`` file that turned out to not be an InnoSetup EXE.
    * If I remember correctly, ``.dll`` files are just PE-format binaries
      without an entry point, so anything that checks the correctness of the
      ``.exe`` portion of a self-extractor should also work on a DLL.
**.deb packages:**
    Either confirm that p7zip is extracting everything or switch to a tool
    which *will* catch corruption in more than just the ``data.tar.gz`` portion
    of the package and then use p7zip as a fallback.
**.tar archives with incorrect extensions:**
    Ensure that a warning is raised if a ``.tar`` file's extension doesn't
    match the kind of compression used. (I've actually seen this in the wild.)

    More generally, I want to double-check the extension-header correspondence
    on everything and prefer to identify by header rather than extension
    whenever feasible.
**Zip files with backslashes in paths:**
    Info-ZIP currently complains about these but then does the same fix-ups
    that tools like WinZIP do, resulting in failures that are related not to
    corruption, but to a non-standard use of the format.
**CD/DVD images:**
    While it'd inherently have to be Linux-specific, mounting the CD image via
    CDEmu_ and then checking all the files within would be a good start which
    supports over a dozen image formats.

    * For ``.iso`` files, I'll also want to try dvdisaster_ in case the image
      has had ECC applied.
    * Beyond that, I need to look into whether anyone has written a fsck-like
      tool for CD/DVD images.

**Chiptunes and MOD files:**
    When I have time, I want to track down or write tools which can catch
    corruption in chiptunes and sequenced music formats.

    ffmpeg's built-in support for libgme and libmodplug loaders is unsuitable
    because it wastes too much time rendering them to an audio stream when all
    that's needed is an integrity check.
**Recursive/Strict Mode:**
    I want to add an option which will unpack archives (rather than merely
    testing them) and check the files within for corruption. (Useful for
    catching cases where a file got corrupted in the past, then you archived it
    without first checking it.)
**JPEG:**
    Identify suspicious horizontal stripes of near-identical pixels at the
    bottom of JPEG files that load properly.
    `[1] <https://www.reddit.com/r/csharp/comments/1fq46h/how_to_detect_partially_corrupt_images/>`_
    (There is a suitable test image at https://superuser.com/q/276154)

**Images:**
    Check for suspicious blocks of ``00`` or ``FF`` values in images
    that load properly. (I'll probably wait to wait for the Rust port for
    performance reasons.)
**Documents:**
    I *want* to verify ``.chm``, ``.doc``, ``.djvu``, ``.mobi``/``.prc``,
    ``.ps``, and ``.rtf`` files but I'm having trouble tracking down utilities
    which can be easily set up to serve as an integrity check.

    * ``.doc`` will require a file header check, because, in addition to being
      used by Microsoft Word, it was also commonly used to mean ``.txt`` in the
      MS-DOS era.

    * I need to check whether any of the tools listed at
      https://unix.stackexchange.com/a/312356/28019 can be pressed into service
      for checking for corruption in RTF files and, if so, which is best.
**Fonts:**
    I need to research what can be checked about these and what tools exist.
**MIDI:**
    When I have time, I want to see whether it's possible to write enough of
    a well-formedness check for MIDI's SMF on-disk format to be worthwhile.
**XML:**
    Look into options for doing schema validation on untrusted XML safely.
**Source Code:**
    While source code doesn't have checksums, it'd be nice to
    at least use parsers to check for syntax errors in HTML, CSS, SVG,
    JavaScript, C, C++, and x86 assembly language source code.

    * For a more advanced option, I could check HTML files first to see if they
      contain `subresource integrity`_ hashes for any of the files associated
      with them.

.. _CDEmu: https://cdemu.sourceforge.io/
.. _Dash: https://kapeli.com/dash
.. _diffoscope: https://diffoscope.org/
.. _dvdisaster: https://en.wikipedia.org/wiki/Dvdisaster
.. _defusedxml: https://pypi.org/project/defusedxml/
.. _ElementTree: https://docs.python.org/3/library/xml.etree.elementtree.html
.. _Mono: https://www.mono-project.com/
.. _subresource integrity: https://developer.mozilla.org/en-US/docs/Web/Security/Subresource_Integrity
.. _Zeal: https://zealdocs.org/
