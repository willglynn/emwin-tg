use crate::Error;
use std::borrow::Cow;
use std::io::Read;

/// A data product from an EMWIN archive.
#[derive(Debug)]
pub struct Product {
    /// The filename of the data product.
    pub filename: String,
    /// The binary contents of the data product.
    pub contents: Vec<u8>,
}

impl Product {
    /// The expected MIME type of this product, if known.
    pub fn mime_type(&self) -> Option<&'static str> {
        Some(match self.filename.rsplit('.').next().unwrap() {
            ".TXT" => "text/plain",
            ".GIF" => "image/gif",
            ".JPG" => "image/jpeg",
            ".PNG" => "image/png",
            _ => return None,
        })
    }

    pub fn string_contents(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.contents)
    }

    pub fn into_string_lossy(self) -> String {
        // Assume it's valid UTF-8
        match String::from_utf8(self.contents) {
            Ok(string) => string,
            Err(e) => {
                // That's surprising
                log::debug!("{} was not valid UTF-8; converting lossily", self.filename);

                // Be lossy in the laziest way possible
                // (This checks it a second time _and_ copies it, instead of converting in place)
                String::from_utf8_lossy(&e.into_bytes()).into_owned()
            }
        }
    }

    pub(crate) fn new(file: zip::result::ZipResult<zip::read::ZipFile>) -> Result<Self, Error> {
        let mut file = file?;

        let mut contents = vec![0u8; file.size().clamp(0, 8 << 20) as usize];
        file.read_exact(&mut contents)
            .map_err(zip::result::ZipError::Io)?;

        let filename = file.name().to_uppercase();

        if filename.ends_with(".ZIP") {
            // Recurse
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&contents))?;
            if archive.len() != 1 {
                Err(Error::ArchiveMember(filename))
            } else {
                Product::new(archive.by_index(0))
            }
        } else {
            Ok(Product { filename, contents })
        }
    }
}
