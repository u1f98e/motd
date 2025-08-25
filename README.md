# motd
A very simple message of the day printer.

Reads entries from a file at `~/.config/motd.conf`, or as specified by the
environment variable `MOTD_FILE`. Entries are delimited with a `%` character. A
random entry will be picked and printed in a random color from this file.

Place in your `.bashrc` or appropriate shell config file for fun.
