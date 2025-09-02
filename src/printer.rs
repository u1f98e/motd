use std::{io::Write, path::Path};

use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{COLOR_LIGHTNESS_LOWER, COLOR_LIGHTNESS_UPPER, Entry, parse::Token};

#[cfg(feature = "image")]
const DEFAULT_IMG_HEIGHT: u32 = 8;

#[derive(Debug, Clone)]
pub struct PrinterConfig {
    pub debug: bool,
    #[cfg(feature = "image")]
    pub img_height: Option<u32>,
    #[cfg(feature = "image")]
    pub img_width: Option<u32>,
}

pub struct MessagePrinter {
    config: PrinterConfig,
}

impl MessagePrinter {
    pub fn new(config: PrinterConfig) -> Self {
        Self { config }
    }

    fn print_formatted_text(&self, msg: &str, color_spec: &termcolor::ColorSpec) {
        if !msg.is_empty() {
            let mut stdout = StandardStream::stdout(ColorChoice::Auto);
            let _ = stdout.set_color(color_spec);
            let _ = writeln!(&mut stdout, "{}", msg.trim());
        }
    }

    fn print_image(&self, _path: &Path) {
        #[cfg(feature = "image")]
        {
            let conf = viuer::Config {
                transparent: true,
                height: Some(self.config.img_height.unwrap_or(DEFAULT_IMG_HEIGHT)),
                width: self.config.img_width,
                absolute_offset: false,
                ..Default::default()
            };

            if let Err(e) = viuer::print_from_file(_path, &conf) {
                println!("🖼️");
                if self.config.debug {
                    eprintln!("motd: Error displaying image {}: {}", _path.display(), e);
                }
            }
        }

        #[cfg(not(feature = "image"))]
        println!("🖼️");
    }

    pub fn process_entry(&self, entry: Entry) {
        let mut color = ColorSpec::new();
        color.set_fg(Some(crate::color::random_color(
            COLOR_LIGHTNESS_LOWER,
            COLOR_LIGHTNESS_UPPER,
        )));

        let tokens = match crate::parse::parse_message(&entry.msg) {
            Ok(t) => t,
            Err(e) => {
                if self.config.debug {
                    eprintln!(
                        "motd: Error parsing entry at line {}: {}",
                        entry.line_number, e
                    );
                }
                return;
            }
        };

        for token in tokens {
            match token {
                Token::Text(text) => self.print_formatted_text(&text, &color),
                Token::Resource(path) => self.print_image(Path::new(&path)),
            }
        }
    }
}
