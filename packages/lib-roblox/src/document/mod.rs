use std::sync::{Arc, RwLock};

use rbx_dom_weak::WeakDom;
use rbx_xml::{
    DecodeOptions as XmlDecodeOptions, DecodePropertyBehavior as XmlDecodePropertyBehavior,
    EncodeOptions as XmlEncodeOptions, EncodePropertyBehavior as XmlEncodePropertyBehavior,
};

mod error;
mod format;
mod kind;

pub use error::*;
pub use format::*;
pub use kind::*;

pub type DocumentResult<T> = Result<T, DocumentError>;

/**
    A container for [`rbx_dom_weak::WeakDom`] that also takes care of
    reading and writing different kinds and formats of roblox files.

    ```rust ignore
    // Reading a document from a file

    let file_path = PathBuf::from("place-file.rbxl");
    let file_contents = std::fs::read(&file_path)?;

    let document = Document::from_bytes_auto(file_contents)?;

    // Writing a document to a file

    let file_path = PathBuf::from("place-file")
        .with_extension(document.extension()?);

    std::fs::write(&file_path, document.to_bytes()?)?;
    ```
*/
#[derive(Debug, Clone)]
pub struct Document {
    kind: DocumentKind,
    format: DocumentFormat,
    dom: Arc<RwLock<WeakDom>>,
}

impl Document {
    /**
        Gets the canonical file extension for a given kind and
        format of document, which will follow this chart:

        | Kind  | Format | Extension |
        |:------|:-------|:----------|
        | Place | Binary | `rbxl`    |
        | Place | Xml    | `rbxlx`   |
        | Model | Binary | `rbxm`    |
        | Model | Xml    | `rbxmx`   |
        | ?     | ?      | None      |

        The last entry here signifies any kind of internal document kind
        or format variant, which should not be used outside of this crate.

        As such, if it is known that no internal specifier is being
        passed here, the return value can be safely unwrapped.
    */
	#[rustfmt::skip]
    pub fn canonical_extension(kind: DocumentKind, format: DocumentFormat) -> Option<&'static str> {
        match (kind, format) {
            (DocumentKind::Place, DocumentFormat::Binary) => Some("rbxl"),
            (DocumentKind::Place, DocumentFormat::Xml)    => Some("rbxlx"),
            (DocumentKind::Model, DocumentFormat::Binary) => Some("rbxm"),
            (DocumentKind::Model, DocumentFormat::Xml)    => Some("rbxmx"),
            _ => None,
        }
    }

    fn from_bytes_inner(bytes: impl AsRef<[u8]>) -> DocumentResult<(DocumentFormat, WeakDom)> {
        let bytes = bytes.as_ref();
        let format = DocumentFormat::from_bytes(bytes).ok_or(DocumentError::UnknownFormat)?;
        let dom = match format {
            DocumentFormat::InternalRoot => Err(DocumentError::InternalRootReadWrite),
            DocumentFormat::Binary => rbx_binary::from_reader(bytes)
                .map_err(|err| DocumentError::ReadError(err.to_string())),
            DocumentFormat::Xml => {
                let xml_options = XmlDecodeOptions::new()
                    .property_behavior(XmlDecodePropertyBehavior::ReadUnknown);
                rbx_xml::from_reader(bytes, xml_options)
                    .map_err(|err| DocumentError::ReadError(err.to_string()))
            }
        }?;
        Ok((format, dom))
    }

    /**
        Decodes and creates a new document from a byte buffer.

        This will automatically handle and detect if the document should be decoded
        using a roblox binary or roblox xml format, and if it is a model or place file.

        Note that detection of model vs place file is heavily dependent on the structure
        of the file, and a model file with services in it will detect as a place file, so
        if possible using [`Document::from_bytes`] with an explicit kind should be preferred.
    */
    pub fn from_bytes_auto(bytes: impl AsRef<[u8]>) -> DocumentResult<Self> {
        let (format, dom) = Self::from_bytes_inner(bytes)?;
        let kind = DocumentKind::from_weak_dom(&dom).ok_or(DocumentError::UnknownKind)?;
        Ok(Self {
            kind,
            format,
            dom: Arc::new(RwLock::new(dom)),
        })
    }

    /**
        Decodes and creates a new document from a byte buffer.

        This will automatically handle and detect if the document
        should be decoded using a roblox binary or roblox xml format.

        Note that passing [`DocumentKind`] enum values other than [`DocumentKind::Place`] and
        [`DocumentKind::Model`] is possible but should only be done within the `lune-roblox` crate.
    */
    pub fn from_bytes(bytes: impl AsRef<[u8]>, kind: DocumentKind) -> DocumentResult<Self> {
        let (format, dom) = Self::from_bytes_inner(bytes)?;
        Ok(Self {
            kind,
            format,
            dom: Arc::new(RwLock::new(dom)),
        })
    }

    /**
        Encodes the document as a vector of bytes, to
        be written to a file or sent over the network.

        This will use the same format that the document was created
        with, meaning if the document is a binary document the output
        will be binary, and vice versa for xml and other future formats.
    */
    pub fn to_bytes(&self) -> DocumentResult<Vec<u8>> {
        self.to_bytes_with_format(self.format)
    }

    /**
        Encodes the document as a vector of bytes, to
        be written to a file or sent over the network.
    */
    pub fn to_bytes_with_format(&self, format: DocumentFormat) -> DocumentResult<Vec<u8>> {
        let dom = self.dom.try_read().expect("Failed to lock dom");
        let mut bytes = Vec::new();
        match format {
            DocumentFormat::InternalRoot => Err(DocumentError::InternalRootReadWrite),
            DocumentFormat::Binary => rbx_binary::to_writer(&mut bytes, &dom, &[dom.root_ref()])
                .map_err(|err| DocumentError::WriteError(err.to_string())),
            DocumentFormat::Xml => {
                let xml_options = XmlEncodeOptions::new()
                    .property_behavior(XmlEncodePropertyBehavior::WriteUnknown);
                rbx_xml::to_writer(&mut bytes, &dom, &[dom.root_ref()], xml_options)
                    .map_err(|err| DocumentError::WriteError(err.to_string()))
            }
        }?;
        Ok(bytes)
    }

    /**
        Gets the kind this document was created with.
    */
    pub fn kind(&self) -> DocumentKind {
        self.kind
    }

    /**
        Gets the format this document was created with.
    */
    pub fn format(&self) -> DocumentFormat {
        self.format
    }

    /**
        Gets the file extension for this document.

        Note that this will return `None` for an internal root
        document, otherwise it will always return `Some`.

        As such, if it is known that no internal root document is
        being used here, the return value can be safely unwrapped.
    */
    pub fn extension(&self) -> Option<&'static str> {
        Self::canonical_extension(self.kind, self.format)
    }

    /**
        Gets the underlying weak dom for this document.
    */
    pub fn dom(&self) -> Arc<RwLock<WeakDom>> {
        Arc::clone(&self.dom)
    }
}
