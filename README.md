# unciv: An Uncivilized File Extractor for Civilization: Call to Power

unciv is an extractor for some proprietary data archives from the 1999
Civilization spin-off [Civilization: Call to Power.](https://en.wikipedia.org/wiki/Civilization:_Call_to_Power).

You can read more, or download precompiled binaries on unciv's website:
https://davidgow.net/hacks/unciv.html

In particular, it extracts the game's zfs archives, which contain the majority
of the miscellaneous graphics (everything except tiles, sprites, and a few
things like mouse cursors), as well as the sounds.

unciv also converts the game's rim graphics format to PNG, including both the
555 and 565 variants.

## Usage

Simply run
```
cargo run <path to .zfs file>
```

The contents of the .zfs file will be extracted into the current directory.

Any file in the archive with the extension ``rim`` will be converted to a .png
when extracted.

If you wish for the original timestamps to be preserved, and you're running a
nightly build of Rust with the ``file_set_times`` supported, you can use
```
cargo run --features set-timestamps <path to .zfs file>
```

## Building

unciv depends on the byteorder (v1.3+) and png (v0.15) crates. Note that png has
an absolute boatload of dependencies of its own, so you'll need to get those
via cargo or some other means.

It's possible to build unciv with the rustc version included with Debian, just
install
```
sudo apt install rustc cargo librust-byteorder-dev librust-png+deflate-dev
```

You can use Debian's packaged crates by following the [Debian Wiki](https://wiki.debian.org/Rust).


Have fun!
— David
