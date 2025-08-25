use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use rand::Rng;
use termcolor::{ColorSpec, StandardStream, WriteColor};

struct EntrySeeker<R: Read + Seek> {
    reader: BufReader<R>,
    positions: Vec<usize>,
    delimiter: u8,
}

const DELIMITER: u8 = b'%';

impl<R> EntrySeeker<R>
where
    R: Read + Seek,
{
    pub fn new(read: R, delimiter: u8) -> io::Result<EntrySeeker<R>> {
        let mut reader = BufReader::new(read);
        let mut positions = Vec::new();
        let mut current_pos = 0;
        let mut _buf = Vec::new();
        loop {
            let count = reader.read_until(delimiter, &mut _buf)?;
            if count == 0 {
                break;
            }
            positions.push(current_pos);
            current_pos += count;
        }

        Ok(EntrySeeker {
            reader,
            positions,
            delimiter,
        })
    }

    pub fn count(&self) -> usize {
        self.positions.len()
    }

    pub fn get_line(&mut self, index: usize) -> io::Result<String> {
        if self.positions.is_empty() {
            return Ok(String::new());
        }

        let pos = self
            .positions
            .get(index)
            .expect("index for line should be in range");
        self.reader.seek(SeekFrom::Start(*pos as u64))?;

        let mut buf = Vec::new();
        self.reader.read_until(self.delimiter, &mut buf)?;
        let mut entry =
            String::from_utf8(buf).expect(&format!("line {index} is not a valid utf8 string"));
        if entry.len() > 0 {
            entry.truncate(entry.len() - 1);
        }

        Ok(entry.trim().to_owned())
    }
}

fn msg_file_path() -> PathBuf {
    std::env::var("MOTD_FILE")
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|_| {
            PathBuf::from(dirs::config_local_dir().unwrap_or_default()).join("motd.conf")
        })
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let chroma = (1. - f32::abs(2. * l - 1.)) * s;
    let h_prime = h * 6.; // H' = H / 60deg
    let x = chroma * (1. - f32::abs(f32::rem_euclid(h_prime, 2.) - 1.)); // X = C * (1 - |H' mod 2 - 1|)
    let (r1, g1, b1) = if 0. <= h_prime && h_prime < 1. {
        (chroma, x, 0.)
    } else if h_prime < 2. {
        (x, chroma, 0.)
    } else if h_prime < 3. {
        (0., chroma, x)
    } else if h_prime < 4. {
        (0., x, chroma)
    } else if h_prime < 5. {
        (x, 0., chroma)
    } else {
        (chroma, 0., x)
    };

    let m = l - (chroma / 2.);
    let r = (r1 + m) * 255.;
    let g = (g1 + m) * 255.;
    let b = (b1 + m) * 255.;
    (r as u8, g as u8, b as u8)
}

/// Returns a [termcolor::Color] with a random hue, full saturation, and a lightness
/// between the provided `lightness_lower` and `lightness_upper` bounds (minimum 0.0, maximum 1.0)
fn random_color(lightness_lower: f32, lightness_upper: f32) -> termcolor::Color {
    let mut rng = rand::thread_rng();
    let (r, g, b) = hsl_to_rgb(
        rng.gen_range(0.0..1.0),
        1.0,
        rng.gen_range(lightness_lower..lightness_upper),
    );
    termcolor::Color::Rgb(r, g, b)
}

fn main() -> io::Result<()> {
    let path = msg_file_path();
    let msg_file = match File::open(&path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!(
                "WARN: Message file '{}' does not exist, creating an empty file.",
                path.display()
            );
            File::create_new(&path).unwrap_or_else(|e2| {
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
    };

    let mut lines = EntrySeeker::new(msg_file, DELIMITER).unwrap();
    if lines.count() == 0 {
        return Ok(());
    }

    let index = rand::thread_rng().gen_range(0..lines.count());
    let msg = lines.get_line(index)?;

    let mut stdout = StandardStream::stdout(termcolor::ColorChoice::Auto);
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(random_color(0.5, 0.9))));
    let _ = writeln!(&mut stdout, "{}", msg.trim());

    Ok(())
}
