use std::path::Path;

use rbx_dom_weak::WeakDom;

use crate::shared::instance::class_is_a_service;

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
        Try to detect a document kind specifier from a weak dom.

        Returns `None` if the given dom is empty and as such can not have its kind inferred.
    */
    pub fn from_weak_dom(dom: &WeakDom) -> Option<Self> {
        let mut has_top_level_child = false;
        let mut has_top_level_service = false;
        for child_ref in dom.root().children() {
            if let Some(child_inst) = dom.get_by_ref(*child_ref) {
                has_top_level_child = true;
                if class_is_a_service(&child_inst.class).unwrap_or(false) {
                    has_top_level_service = true;
                    break;
                }
            }
        }
        if has_top_level_service {
            Some(Self::Place)
        } else if has_top_level_child {
            Some(Self::Model)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rbx_dom_weak::InstanceBuilder;

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

    #[test]
    fn from_weak_dom() {
        let empty = WeakDom::new(InstanceBuilder::new("Instance"));
        assert_eq!(DocumentKind::from_weak_dom(&empty), None);

        let with_services = WeakDom::new(
            InstanceBuilder::new("Instance")
                .with_child(InstanceBuilder::new("Workspace"))
                .with_child(InstanceBuilder::new("ReplicatedStorage")),
        );
        assert_eq!(
            DocumentKind::from_weak_dom(&with_services),
            Some(DocumentKind::Place)
        );

        let with_children = WeakDom::new(
            InstanceBuilder::new("Instance")
                .with_child(InstanceBuilder::new("Model"))
                .with_child(InstanceBuilder::new("Part")),
        );
        assert_eq!(
            DocumentKind::from_weak_dom(&with_children),
            Some(DocumentKind::Model)
        );

        let with_mixed = WeakDom::new(
            InstanceBuilder::new("Instance")
                .with_child(InstanceBuilder::new("Workspace"))
                .with_child(InstanceBuilder::new("Part")),
        );
        assert_eq!(
            DocumentKind::from_weak_dom(&with_mixed),
            Some(DocumentKind::Place)
        );
    }
}
