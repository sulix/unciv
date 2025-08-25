# unciv: An Uncivilized File Extractor for Civilization: Call to Power

unciv is an extractor for some proprietary data archives from the 1999
Civilization spin-off [Civilization: Call to Power.](https://en.wikipedia.org/wiki/Civilization:_Call_to_Power).

You can read more, or download precompiled binaries on unciv's website:
https://davidgow.net/hacks/unciv.html

In particular, it extracts the game's zfs archives, which contain the majority
of the miscellaneous graphics (everything except tiles, sprites, and a few
things like mouse cursors), as well as the sounds.

unciv can also convert the game's rim graphics format to PNG, including both
the 555 and 565 variants.

## Usage

Simply run
```
cargo run <path to .zfs file>
```

The contents of the .zfs file will be extracted into the current directory.

To convert .rim files to .png, you will need the ``png`` crate, and to enable
the ``png`` feature, with:
```
cargo run --features png <path to .zfs file>
```

Any file in the archive with the extension ``rim`` will then be converted to a
.png when extracted.

If you wish for the original timestamps to be preserved, and you're running
Rust v1.75 or newer (or a nightly build of Rust with the ``file_set_times``
feature supported), you can use
```
cargo run --features set-timestamps <path to .zfs file>
```

The ``png`` and ``set-timestamps`` features can be combined.

## Usage as a Library

``unciv`` can also be used as a Rust library to parse and read ``.zfs`` files,
and the ``rim`` files within them.

To load a ``.zfs`` file, you'll need to use the ``ZfsFile::from_stream()``
function. The resulting ``ZfsFile`` struct has a ``Vec`` of ``ZfsEntry``
structs, upon which either ``read_data()`` can be called to retrieve the
contents of the file, or ``read_rim_image()`` to retrieve it in the form of a
``RimImage`` struct.

The ``RimImage`` struct contains information like the width and height, as
well as the raw data. Alternatively, the ``to_rgba_bytes()`` function can be
used to convert the data to 32-bit RGBA format.

You can get the library's documentation using ``cargo doc``, or you can look
at ``src/main.rs`` — the source code for the command-line utility — which
serves as a good example of the API.

## Building

unciv depends on the png (v0.15) crate. Note that png has an absolute boatload
of dependencies of its own, so you'll need to get those via cargo or some other
means.

It's possible to build unciv with the rustc version included with Debian, just
install
```
sudo apt install rustc cargo librust-png+deflate-dev
```

You can use Debian's packaged crates by following the [Debian Wiki](https://wiki.debian.org/Rust).

Note, however, that version 0.15 of png is only available in Debian bullseye.
This does mean that building ``unciv`` on newer versions of Debian requires
updating the ``png`` dependency to version 0.17 (even if the ``png`` feature is
disabled, as the generation of ``Cargo.lock`` apparently requires the version
to exist in the repository even if it isn't used). This can be done by just
changing the version in ``Cargo.toml``, and replacing ``RGBA`` with ``Rgba``
in ``src/main.rs``, which is the only change required.

Have fun!
— David
