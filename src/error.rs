/// An error which occurred while retrieving from an EMWIN TG stream.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A failure occurred during an HTTP exchange
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    /// The retrieved archive could not be processed
    #[error("archive format error: {0}")]
    ArchiveFormat(#[from] zip::result::ZipError),
    /// An entry within the archive could not be processed
    #[error("inner archive format error in {0:?}")]
    ArchiveMember(String),
}
