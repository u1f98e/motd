# motd
A very simple message of the day printer.

Reads entries from a file at `~/.config/motd.conf`, or as specified by the
environment variable `MOTD_FILE`. Entries are delimited with a `%` character. A
random entry will be picked from this file and printed in a random color.

Place in your `.bashrc` or appropriate shell config file for fun.

```
Usage: motd [options]
  -e, --entry <NUM>   Print entry NUM instead of a random entry.
      --debug         Print error messages instead of suppressing them.
      --validate      Check message file for parsing errors.

      (`image` feature only)
      --img-height    Set the height in columns to use for images, defaults to 8.
      --img-width     Manually set the width for images, preserves the aspect ratio by default.
```

### Syntax
Entries in the `motd.conf` file are delimited by a `%` character,
and will have leading and trailing whitespace trimmed.

If the `image` feature has been enabled, images can be embedded into entries
using square brackets surrounding a path: `[/path/to/file.png]`. Images
render best on terminals with support for the Kitty graphics protocol,
though currently only [ghostty has support](https://github.com/atanunq/viuer/issues/70).

Sixel support may be enabled with the `image-sixel` feature, which requires
that the `libsixel` library be present on your system.

Delimiters and brackets can be escaped with `\`: `hello \% world \\ :\]`
