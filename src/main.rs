mod color;
mod parse;
mod printer;

use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use rand::Rng;

use crate::parse::Token;
use crate::printer::{MessagePrinter, PrinterConfig};

const ENTRY_DELIMITER: u8 = b'%';
const ENTRY_DELIMITER_CHAR: char = ENTRY_DELIMITER as char;
const COLOR_LIGHTNESS_LOWER: f32 = 0.5;
const COLOR_LIGHTNESS_UPPER: f32 = 0.9;

#[derive(Debug)]
struct SeekPos {
    pub start_pos: usize,
    pub len: usize,
    pub line_number: u32,
}

#[derive(Default, Debug)]
struct Entry {
    pub msg: String,
    pub line_number: u32,
    pub entry_number: usize,
}

struct EntrySeeker<R: Read + Seek> {
    reader: BufReader<R>,
    entry_positions: Vec<SeekPos>,
}

impl<R> EntrySeeker<R>
where
    R: Read + Seek,
{
    pub fn new(read: R) -> io::Result<EntrySeeker<R>> {
        let mut reader = BufReader::new(read);
        let mut entries = Vec::new();
        let mut entry_pos = 0;
        let mut entry_len = 0;
        let mut current_line = 1;
        let mut entry_line = None;
        let mut buf = [0u8; 1024];
        loop {
            let count = reader.read(&mut buf)?;
            if count == 0 {
                break;
            }

            let mut escape = false;
            for byte in buf.iter().take(count) {
                entry_len += 1;

                if entry_line.is_none() && !byte.is_ascii_whitespace() {
                    entry_line = Some(current_line);
                }

                match byte {
                    b'\\' => {
                        escape = true;
                    }
                    b'\n' => {
                        current_line += 1;
                        escape = false;
                    }
                    &ENTRY_DELIMITER => {
                        if !escape {
                            entries.push(SeekPos {
                                start_pos: entry_pos,
                                len: entry_len,
                                line_number: entry_line.unwrap_or(current_line),
                            });
                            entry_pos += entry_len;
                            entry_len = 0;
                            entry_line = None;
                        }
                        escape = false
                    }
                    _ => escape = false,
                }
            }
        }

        Ok(EntrySeeker {
            reader,
            entry_positions: entries,
        })
    }

    pub fn count(&self) -> usize {
        self.entry_positions.len()
    }

    pub fn entries(self) -> Entries<R> {
        Entries::new(self)
    }

    pub fn entry_positions(&'_ self) -> core::slice::Iter<'_, SeekPos> {
        self.entry_positions.iter()
    }

    pub fn get_entry(&mut self, index: usize) -> io::Result<Entry> {
        if self.entry_positions.is_empty() {
            return Ok(Entry::default());
        }

        let seek_pos = self
            .entry_positions
            .get(index)
            .expect("index for line should be in range");
        self.reader
            .seek(SeekFrom::Start(seek_pos.start_pos as u64))?;

        let mut buf = vec![0u8; seek_pos.len];
        self.reader.read_exact(&mut buf)?;
        let mut msg = String::from_utf8(buf).expect(&format!(
            "Entry on line {} is not a valid utf8 string",
            seek_pos.line_number
        ));
        if msg.len() > 0 {
            msg.truncate(msg.len() - 1); // Chop off delimiter
        }
        msg = msg.trim().to_owned();

        Ok(Entry {
            msg,
            line_number: seek_pos.line_number,
            entry_number: index,
        })
    }
}

struct Entries<R>
where
    R: Read + Seek,
{
    seeker: EntrySeeker<R>,
    index: usize,
}

impl<R> Entries<R>
where
    R: Read + Seek,
{
    pub fn new(seeker: EntrySeeker<R>) -> Self {
        Self { seeker, index: 0 }
    }
}

impl<R> Iterator for Entries<R>
where
    R: Read + Seek,
{
    type Item = Result<Entry, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.seeker.count() {
            return None;
        }
        let result = Some(self.seeker.get_entry(self.index));
        self.index += 1;
        result
    }

    fn count(self) -> usize {
        self.seeker.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index = n;
        self.next()
    }
}

fn msg_file_path() -> PathBuf {
    std::env::var("MOTD_FILE")
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|_| {
            PathBuf::from(dirs::config_local_dir().unwrap_or_default()).join("motd.conf")
        })
}

fn print_help() {
    println!(
        "Usage: motd [options]
  -e, --entry <NUM>   Print entry NUM instead of a random entry.
      --line <NUM>    Print entry on line NUM instead of a random entry (debugging only).
      --debug         Print error messages instead of suppressing them.
      --validate      Check message file for parsing errors."
    );
    #[cfg(feature = "image")]
    println!(
        "
      --img-height    Set the height in columns to use for images, defaults to 8.
      --img-width     Manually set the width for images, preserves the aspect ratio by default."
    );
}

#[derive(Default)]
struct CliArgs {
    pub debug: bool,
    pub validate: bool,
    pub entry: Option<u32>,
    pub line_num: Option<u32>,
    #[cfg(feature = "image")]
    pub img_height: Option<u32>,
    #[cfg(feature = "image")]
    pub img_width: Option<u32>,
}

impl CliArgs {
    pub fn from_args(mut args: std::env::Args) -> Self {
        let mut value = Self::default();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" => print_help(),
                "--debug" => value.debug = true,
                "--validate" => value.validate = true,
                "-e" | "--entry" => {
                    let Some(entry) = args.next().and_then(|a| a.parse().ok()) else {
                        eprintln!("motd: --entry option requires a valid entry number.");
                        std::process::exit(1);
                    };

                    value.entry = Some(entry);
                }
                "--line" => {
                    let Some(entry) = args.next().and_then(|a| a.parse().ok()) else {
                        eprintln!("motd: --line option requires a valid line number.");
                        std::process::exit(1);
                    };

                    println!("{entry:?}");
                    value.line_num = Some(entry);
                }
                #[cfg(feature = "image")]
                "--img-height" => {
                    let Some(entry) = args.next().map(|a| a.parse().ok()) else {
                        eprintln!("motd: --img-height option requires a valid size.");
                        std::process::exit(1);
                    };

                    value.img_height = entry;
                }
                #[cfg(feature = "image")]
                "--img-width" => {
                    let Some(entry) = args.next().map(|a| a.parse().ok()) else {
                        eprintln!("motd: --img-width option requires a valid size.");
                        std::process::exit(1);
                    };

                    value.img_width = entry;
                }
                _ => (),
            }
        }

        value
    }
}

fn open_msg_file(path: &Path) -> File {
    match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!(
                "motd: Message file '{}' does not exist, creating an empty file.",
                path.display()
            );
            File::create_new(path).unwrap_or_else(|e2| {
                eprintln!(
                    "motd: failed to create new message file '{}': {}",
                    path.display(),
                    e2
                );
                std::process::exit(1);
            })
        }
        Err(e) => {
            eprintln!(
                "motd: failed to open message file '{}': {}",
                path.display(),
                e
            );
            std::process::exit(1);
        }
    }
}

fn validate_file<R>(entry_seeker: EntrySeeker<R>) -> io::Result<()>
where
    R: std::io::Seek + std::io::Read,
{
    for entry in entry_seeker.entries() {
        let entry = match entry {
            Ok(en) => en,
            Err(e) => {
                eprintln!("Validation error: failed to read entry: {e}");
                std::process::exit(1);
            }
        };

        let tokens = match parse::parse_message(&entry.msg) {
            Ok(tokens) => tokens,
            Err(e) => {
                eprintln!("Validation error on line {}: {}", entry.line_number, e);
                std::process::exit(1);
            }
        };

        for token in tokens {
            if let Token::Resource(p) = token {
                let path = Path::new(&p);
                if !path.exists() {
                    eprintln!(
                        "Resource '{}' doesn't exist (from line {})",
                        path.display(),
                        entry.line_number
                    );
                }
            }
        }
    }

    return Ok(());
}

fn main() -> io::Result<()> {
    // Process args
    let args = CliArgs::from_args(std::env::args());
    let msg_path = msg_file_path();
    let msg_file = open_msg_file(&msg_path);

    let mut entry_seeker = EntrySeeker::new(msg_file).unwrap();
    if entry_seeker.count() == 0 {
        if args.debug {
            eprintln!(
                "motd: Message file '{}' does not contain any entries.",
                msg_path.display()
            );
        }
        return Ok(());
    }

    if args.validate {
        // Do file validation instead
        return validate_file(entry_seeker);
    }

    let entry_num = if let Some(entry_num) = args.entry {
        if entry_num as usize >= entry_seeker.count() {
            eprintln!(
                "Requested entry exceeds entry count ({})",
                entry_seeker.count()
            );
            std::process::exit(1);
        }
        entry_seeker.get_entry(
            entry_num
                .try_into()
                .expect("Should have less than u32 entries"),
        )?
    } else if let Some(line_num) = args.line_num {
        // Find the entry which contains line_num, or saturate on the last entry
        let mut entry_num = 0;
        for (i, pos) in entry_seeker.entry_positions().enumerate() {
            entry_num = i;
            if pos.line_number >= line_num {
                break;
            }
        }

        if entry_num as usize >= entry_seeker.count() {
            eprintln!("Line number exceeds message file line count");
            std::process::exit(1);
        }
        let entry = entry_seeker.get_entry(entry_num)?;

        if args.debug {
            eprintln!("Line: {}\nEntry: {}", entry.line_number, entry.entry_number);
        }

        entry
    } else {
        let index = rand::thread_rng().gen_range(0..entry_seeker.count());
        entry_seeker.get_entry(index)?
    };

    let printer = MessagePrinter::new(PrinterConfig {
        debug: args.debug,

        #[cfg(feature = "image")]
        img_height: args.img_height,
        #[cfg(feature = "image")]
        img_width: args.img_width,
    });
    printer.process_entry(entry_num);
    Ok(())
}
