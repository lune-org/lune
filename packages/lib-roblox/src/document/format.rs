// Original implementation from Remodel:
// https://github.com/rojo-rbx/remodel/blob/master/src/sniff_type.rs

use std::path::Path;

/**
    A document format specifier.

    Valid variants are the following:

    - `Binary`
    - `Xml`

    Other variants are only to be used for logic internal to this crate.
*/
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DocumentFormat {
    InternalRoot,
    Binary,
    Xml,
}

impl DocumentFormat {
    /**
        Try to convert a file extension into a valid document format specifier.

        Returns `None` if the file extension is not a canonical roblox file format extension.
    */
    pub fn from_extension(extension: impl AsRef<str>) -> Option<Self> {
        match extension.as_ref() {
            "rbxl" | "rbxm" => Some(Self::Binary),
            "rbxlx" | "rbxmx" => Some(Self::Xml),
            _ => None,
        }
    }

    /**
        Try to convert a file path into a valid document format specifier.

        Returns `None` if the file extension of the path
        is not a canonical roblox file format extension.
    */
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        match path
            .as_ref()
            .extension()
            .map(|ext| ext.to_string_lossy())
            .as_deref()
        {
            Some("rbxl") | Some("rbxm") => Some(Self::Binary),
            Some("rbxlx") | Some("rbxmx") => Some(Self::Xml),
            _ => None,
        }
    }

    /**
        Try to detect a document format specifier from file contents.

        Returns `None` if the file contents do not seem to be from a valid roblox file.
    */
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Option<Self> {
        let header = bytes.as_ref().get(0..8)?;

        if header.starts_with(b"<roblox") {
            match header[7] {
                b'!' => Some(Self::Binary),
                b' ' | b'>' => Some(Self::Xml),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn from_extension_binary() {
        assert_eq!(
            DocumentFormat::from_extension("rbxl"),
            Some(DocumentFormat::Binary)
        );

        assert_eq!(
            DocumentFormat::from_extension("rbxm"),
            Some(DocumentFormat::Binary)
        );
    }

    #[test]
    fn from_extension_xml() {
        assert_eq!(
            DocumentFormat::from_extension("rbxlx"),
            Some(DocumentFormat::Xml)
        );

        assert_eq!(
            DocumentFormat::from_extension("rbxmx"),
            Some(DocumentFormat::Xml)
        );
    }

    #[test]
    fn from_extension_invalid() {
        assert_eq!(DocumentFormat::from_extension("csv"), None);
        assert_eq!(DocumentFormat::from_extension("json"), None);
        assert_eq!(DocumentFormat::from_extension("rbx"), None);
        assert_eq!(DocumentFormat::from_extension("rbxn"), None);
        assert_eq!(DocumentFormat::from_extension("xlx"), None);
        assert_eq!(DocumentFormat::from_extension("xmx"), None);
    }

    #[test]
    fn from_path_binary() {
        assert_eq!(
            DocumentFormat::from_path(PathBuf::from("model.rbxl")),
            Some(DocumentFormat::Binary)
        );

        assert_eq!(
            DocumentFormat::from_path(PathBuf::from("model.rbxm")),
            Some(DocumentFormat::Binary)
        );
    }

    #[test]
    fn from_path_xml() {
        assert_eq!(
            DocumentFormat::from_path(PathBuf::from("place.rbxlx")),
            Some(DocumentFormat::Xml)
        );

        assert_eq!(
            DocumentFormat::from_path(PathBuf::from("place.rbxmx")),
            Some(DocumentFormat::Xml)
        );
    }

    #[test]
    fn from_path_invalid() {
        assert_eq!(
            DocumentFormat::from_path(PathBuf::from("data-file.csv")),
            None
        );
        assert_eq!(
            DocumentFormat::from_path(PathBuf::from("nested/path/file.json")),
            None
        );
        assert_eq!(
            DocumentFormat::from_path(PathBuf::from(".no-name-strange-rbx")),
            None
        );
        assert_eq!(
            DocumentFormat::from_path(PathBuf::from("file_without_extension")),
            None
        );
    }

    #[test]
    fn from_bytes_binary() {
        assert_eq!(
            DocumentFormat::from_bytes(b"<roblox!hello"),
            Some(DocumentFormat::Binary)
        );

        assert_eq!(
            DocumentFormat::from_bytes(b"<roblox!"),
            Some(DocumentFormat::Binary)
        );
    }

    #[test]
    fn from_bytes_xml() {
        assert_eq!(
            DocumentFormat::from_bytes(b"<roblox xml:someschemajunk>"),
            Some(DocumentFormat::Xml)
        );

        assert_eq!(
            DocumentFormat::from_bytes(b"<roblox>"),
            Some(DocumentFormat::Xml)
        );
    }

    #[test]
    fn from_bytes_invalid() {
        assert_eq!(DocumentFormat::from_bytes(b""), None);
        assert_eq!(DocumentFormat::from_bytes(b" roblox"), None);
        assert_eq!(DocumentFormat::from_bytes(b"<roblox"), None);
        assert_eq!(DocumentFormat::from_bytes(b"<roblox-"), None);
    }
}
