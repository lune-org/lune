use std::path::Path;

/**
    A document kind specifier.

    Valid variants are the following:

    - `Model`
    - `Place`

    Other variants are only to be used for logic internal to this crate.
*/
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DocumentKind {
    InternalRoot,
    Place,
    Model,
}

impl DocumentKind {
    /**
        Try to convert a file extension into a valid document kind specifier.

        Returns `None` if the file extension is not a canonical roblox file format extension.
    */
    pub fn from_extension(extension: impl AsRef<str>) -> Option<Self> {
        match extension.as_ref() {
            "rbxl" | "rbxlx" => Some(Self::Place),
            "rbxm" | "rbxmx" => Some(Self::Model),
            _ => None,
        }
    }

    /**
        Try to convert a file path into a valid document kind specifier.

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
            Some("rbxl") | Some("rbxlx") => Some(Self::Place),
            Some("rbxm") | Some("rbxmx") => Some(Self::Model),
            _ => None,
        }
    }

    /**
        Try to detect a document kind specifier from file contents.

        Returns `None` if the file contents do not seem to be from a valid roblox file.
    */
    pub fn from_bytes(_bytes: impl AsRef<[u8]>) -> Option<Self> {
        // TODO: Implement this, read comment below
        todo!("Investigate if it is possible to detect document kind from contents")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn from_extension_place() {
        assert_eq!(
            DocumentKind::from_extension("rbxl"),
            Some(DocumentKind::Place)
        );

        assert_eq!(
            DocumentKind::from_extension("rbxlx"),
            Some(DocumentKind::Place)
        );
    }

    #[test]
    fn from_extension_model() {
        assert_eq!(
            DocumentKind::from_extension("rbxm"),
            Some(DocumentKind::Model)
        );

        assert_eq!(
            DocumentKind::from_extension("rbxmx"),
            Some(DocumentKind::Model)
        );
    }

    #[test]
    fn from_extension_invalid() {
        assert_eq!(DocumentKind::from_extension("csv"), None);
        assert_eq!(DocumentKind::from_extension("json"), None);
        assert_eq!(DocumentKind::from_extension("rbx"), None);
        assert_eq!(DocumentKind::from_extension("rbxn"), None);
        assert_eq!(DocumentKind::from_extension("xlx"), None);
        assert_eq!(DocumentKind::from_extension("xmx"), None);
    }

    #[test]
    fn from_path_place() {
        assert_eq!(
            DocumentKind::from_path(PathBuf::from("place.rbxl")),
            Some(DocumentKind::Place)
        );

        assert_eq!(
            DocumentKind::from_path(PathBuf::from("place.rbxlx")),
            Some(DocumentKind::Place)
        );
    }

    #[test]
    fn from_path_model() {
        assert_eq!(
            DocumentKind::from_path(PathBuf::from("model.rbxm")),
            Some(DocumentKind::Model)
        );

        assert_eq!(
            DocumentKind::from_path(PathBuf::from("model.rbxmx")),
            Some(DocumentKind::Model)
        );
    }

    #[test]
    fn from_path_invalid() {
        assert_eq!(
            DocumentKind::from_path(PathBuf::from("data-file.csv")),
            None
        );
        assert_eq!(
            DocumentKind::from_path(PathBuf::from("nested/path/file.json")),
            None
        );
        assert_eq!(
            DocumentKind::from_path(PathBuf::from(".no-name-strange-rbx")),
            None
        );
        assert_eq!(
            DocumentKind::from_path(PathBuf::from("file_without_extension")),
            None
        );
    }

    // TODO: Add tests here for the from_bytes implementation
}
