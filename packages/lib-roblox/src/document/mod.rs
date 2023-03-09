use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use rbx_dom_weak::{types::Ref, WeakDom};
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

    /**
        Decodes and creates a new document from a byte buffer.

        This will automatically handle and detect if the document should be decoded
        using a roblox binary or roblox xml format, and if it is a model or place file.
    */
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, DocumentError> {
        let bytes = bytes.as_ref();
        let kind = DocumentKind::from_bytes(bytes).ok_or(DocumentError::UnknownKind)?;
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
    pub fn to_bytes(&self) -> Result<Vec<u8>, DocumentError> {
        self.to_bytes_with_format(self.format)
    }

    /**
        Encodes the document as a vector of bytes, to
        be written to a file or sent over the network.
    */
    pub fn to_bytes_with_format(&self, format: DocumentFormat) -> Result<Vec<u8>, DocumentError> {
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
        Retrieves the root referent of the underlying weak dom.
    */
    pub fn get_root_ref(&self) -> Ref {
        let dom = self.dom.try_read().expect("Failed to lock dom");
        dom.root_ref()
    }

    /**
        Retrieves all root child referents of the underlying weak dom.
    */
    pub fn get_root_child_refs(&self) -> Vec<Ref> {
        let dom = self.dom.try_read().expect("Failed to lock dom");
        dom.root().children().to_vec()
    }

    /**
        Retrieves a reference to the underlying weak dom.
    */
    pub fn get_dom(&self) -> RwLockReadGuard<WeakDom> {
        self.dom.try_read().expect("Failed to lock dom")
    }

    /**
        Retrieves a mutable reference to the underlying weak dom.
    */
    pub fn get_dom_mut(&mut self) -> RwLockWriteGuard<WeakDom> {
        self.dom.try_write().expect("Failed to lock dom")
    }

    /**
        Consumes the document, returning the underlying weak dom.

        This may panic if the document has been cloned
        and still has another owner in memory.
    */
    pub fn into_dom(self) -> WeakDom {
        let lock = Arc::try_unwrap(self.dom).expect("Document has multiple owners in memory");
        lock.into_inner().expect("Failed to lock dom")
    }
}
