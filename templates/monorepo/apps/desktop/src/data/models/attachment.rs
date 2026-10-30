//! The `Attachment` model.

/// Attachment: one of its variants.
#[derive(Clone, Debug, PartialEq)]
pub enum Attachment {
    /// Image.
    Image {
        /// `url`.
        url: String,
        /// `width`.
        width: u32,
    },
    /// File.
    File {
        /// `url`.
        url: String,
        /// `file_name`.
        file_name: String,
    },
}
