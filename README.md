# motd
A very simple message of the day printer.

Reads lines from a file at `~/.config/motd.conf`, or as specified by the
environment variable `MOTD_FILE`. A random line will be picked and printed
in a random color from this file.

Place in your `.bashrc` or appropriate shell config file for fun.