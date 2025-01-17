use crate::error::ViuResult;
use crate::printer::{adjust_offset, find_best_fit, Printer};
use crate::Config;
use image::{DynamicImage, GenericImageView};
use lazy_static::lazy_static;
use std::io::{BufReader, Read, Write};

#[allow(non_camel_case_types)]
pub struct iTermPrinter {}

lazy_static! {
    static ref ITERM_SUPPORT: bool = check_iterm_support();
}

/// Returns the terminal's support for the iTerm graphics protocol.
pub fn is_iterm_supported() -> bool {
    *ITERM_SUPPORT
}

impl Printer for iTermPrinter {
    fn print(&self, img: &DynamicImage, config: &Config) -> ViuResult<(u32, u32)> {
        let (width, height) = img.dimensions();

        // Transform the dynamic image to a PNG which can be given directly to iTerm
        let mut png_bytes: Vec<u8> = Vec::new();
        let _ = image::codecs::png::PngEncoder::new(&mut png_bytes).encode(
            img.as_bytes(),
            width,
            height,
            img.color(),
        )?;

        print_buffer(img, &png_bytes[..], config)
    }

    fn print_from_file(&self, filename: &str, config: &Config) -> ViuResult<(u32, u32)> {
        let file = std::fs::File::open(filename)?;

        // load the file content
        let mut buf_reader = BufReader::new(file);
        let mut file_content = Vec::new();
        buf_reader.read_to_end(&mut file_content)?;

        let img = image::load_from_memory(&file_content[..])?;
        print_buffer(&img, &file_content[..], config)
    }
}

// This function requires both a DynamicImage, which is used to calculate dimensions,
// and it's raw representation as a file, because that's the data iTerm needs to display it.
fn print_buffer(img: &DynamicImage, img_content: &[u8], config: &Config) -> ViuResult<(u32, u32)> {
    let mut stdout = std::io::stdout();

    adjust_offset(&mut stdout, config)?;

    // Note: find_best_fit is not necessary for iTerm2. It will already fit to the terminal size if
    // no height or width are provided. If only one is provided, it will scale the aspect ratio
    // appropriately. Additionally, it's calculations don't seem to be working properly with iTerm2
    // anyway.
    // TODO: Keeping find_best_fit here anyway just because we need a ViuResult.
    // TODO: Maybe fix find_best_fit instead? It would be more elegant.
    let (w, h) = find_best_fit(&img, config.width, config.height);

    let w_str = match config.width {
        Some(w) => format!("width={};", w),
        None => "".to_string(),
    };

    let h_str = match config.height {
        Some(h) => format!("height={};", h),
        None => "".to_string(),
    };

    writeln!(
        stdout,
        "\x1b]1337;File=inline=1;preserveAspectRatio=1;size={};{}{}:{}\x07",
        img_content.len(),
        w_str,
        h_str,
        base64::encode(img_content)
    )?;
    stdout.flush()?;

    Ok((w, h))
}

// Check if the iTerm protocol can be used
fn check_iterm_support() -> bool {
    if let Ok(term) = std::env::var("TERM_PROGRAM") {
        if term.contains("iTerm") {
            return true;
        }
    }
    false
}
