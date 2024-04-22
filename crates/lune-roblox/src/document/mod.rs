use rbx_dom_weak::{types::Ref as DomRef, InstanceBuilder as DomInstanceBuilder, WeakDom};
use rbx_xml::{
    DecodeOptions as XmlDecodeOptions, DecodePropertyBehavior as XmlDecodePropertyBehavior,
    EncodeOptions as XmlEncodeOptions, EncodePropertyBehavior as XmlEncodePropertyBehavior,
};

mod error;
mod format;
mod kind;
mod postprocessing;

pub use error::*;
pub use format::*;
pub use kind::*;

use postprocessing::*;

use crate::instance::{data_model, Instance};

pub type DocumentResult<T> = Result<T, DocumentError>;

/**
    A container for [`rbx_dom_weak::WeakDom`] that also takes care of
    reading and writing different kinds and formats of roblox files.

    ---

    ### Code Sample #1

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

    ---

    ### Code Sample #2

    ```rust ignore
    // Converting a Document to a DataModel or model child instances
    let data_model = document.into_data_model_instance()?;

    let model_children = document.into_instance_array()?;

    // Converting a DataModel or model child instances into a Document
    let place_doc = Document::from_data_model_instance(data_model)?;

    let model_doc = Document::from_instance_array(model_children)?;
    ```
*/
#[derive(Debug)]
pub struct Document {
    kind: DocumentKind,
    format: DocumentFormat,
    dom: WeakDom,
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
    */
    #[must_use]
	#[rustfmt::skip]
    pub fn canonical_extension(kind: DocumentKind, format: DocumentFormat) -> &'static str {
        match (kind, format) {
            (DocumentKind::Place, DocumentFormat::Binary) => "rbxl",
            (DocumentKind::Place, DocumentFormat::Xml)    => "rbxlx",
            (DocumentKind::Model, DocumentFormat::Binary) => "rbxm",
            (DocumentKind::Model, DocumentFormat::Xml)    => "rbxmx",
        }
    }

    fn from_bytes_inner(bytes: impl AsRef<[u8]>) -> DocumentResult<(DocumentFormat, WeakDom)> {
        let bytes = bytes.as_ref();
        let format = DocumentFormat::from_bytes(bytes).ok_or(DocumentError::UnknownFormat)?;
        let dom = match format {
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

        # Errors

        Errors if the given bytes are not a valid roblox file.
    */
    pub fn from_bytes_auto(bytes: impl AsRef<[u8]>) -> DocumentResult<Self> {
        let (format, dom) = Self::from_bytes_inner(bytes)?;
        let kind = DocumentKind::from_weak_dom(&dom).ok_or(DocumentError::UnknownKind)?;
        Ok(Self { kind, format, dom })
    }

    /**
        Decodes and creates a new document from a byte buffer.

        This will automatically handle and detect if the document
        should be decoded using a roblox binary or roblox xml format.

        # Errors

        Errors if the given bytes are not a valid roblox file or not of the given kind.
    */
    pub fn from_bytes(bytes: impl AsRef<[u8]>, kind: DocumentKind) -> DocumentResult<Self> {
        let (format, dom) = Self::from_bytes_inner(bytes)?;
        Ok(Self { kind, format, dom })
    }

    /**
        Encodes the document as a vector of bytes, to
        be written to a file or sent over the network.

        This will use the same format that the document was created
        with, meaning if the document is a binary document the output
        will be binary, and vice versa for xml and other future formats.

        # Errors

        Errors if the document can not be encoded.
    */
    pub fn to_bytes(&self) -> DocumentResult<Vec<u8>> {
        self.to_bytes_with_format(self.format)
    }

    /**
        Encodes the document as a vector of bytes, to
        be written to a file or sent over the network.

        # Errors

        Errors if the document can not be encoded.
    */
    pub fn to_bytes_with_format(&self, format: DocumentFormat) -> DocumentResult<Vec<u8>> {
        let mut bytes = Vec::new();
        match format {
            DocumentFormat::Binary => {
                rbx_binary::to_writer(&mut bytes, &self.dom, self.dom.root().children())
                    .map_err(|err| DocumentError::WriteError(err.to_string()))
            }
            DocumentFormat::Xml => {
                let xml_options = XmlEncodeOptions::new()
                    .property_behavior(XmlEncodePropertyBehavior::WriteUnknown);
                rbx_xml::to_writer(
                    &mut bytes,
                    &self.dom,
                    self.dom.root().children(),
                    xml_options,
                )
                .map_err(|err| DocumentError::WriteError(err.to_string()))
            }
        }?;
        Ok(bytes)
    }

    /**
        Gets the kind this document was created with.
    */
    #[must_use]
    pub fn kind(&self) -> DocumentKind {
        self.kind
    }

    /**
        Gets the format this document was created with.
    */
    #[must_use]
    pub fn format(&self) -> DocumentFormat {
        self.format
    }

    /**
        Gets the file extension for this document.
    */
    #[must_use]
    pub fn extension(&self) -> &'static str {
        Self::canonical_extension(self.kind, self.format)
    }

    /**
        Creates a `DataModel` instance out of this place document.

        # Errors

        Errors if the document is not a place.
    */
    pub fn into_data_model_instance(mut self) -> DocumentResult<Instance> {
        if self.kind != DocumentKind::Place {
            return Err(DocumentError::IntoDataModelInvalidArgs);
        }

        let dom_root = self.dom.root_ref();

        let data_model_ref = self
            .dom
            .insert(dom_root, DomInstanceBuilder::new(data_model::CLASS_NAME));
        let data_model_child_refs = self.dom.root().children().to_vec();

        for child_ref in data_model_child_refs {
            if child_ref != data_model_ref {
                self.dom.transfer_within(child_ref, data_model_ref);
            }
        }

        Ok(Instance::from_external_dom(&mut self.dom, data_model_ref))
    }

    /**
        Creates an array of instances out of this model document.

        # Errors

        Errors if the document is not a model.
    */
    pub fn into_instance_array(mut self) -> DocumentResult<Vec<Instance>> {
        if self.kind != DocumentKind::Model {
            return Err(DocumentError::IntoInstanceArrayInvalidArgs);
        }

        let dom_child_refs = self.dom.root().children().to_vec();

        let root_child_instances = dom_child_refs
            .into_iter()
            .map(|child_ref| Instance::from_external_dom(&mut self.dom, child_ref))
            .collect();

        Ok(root_child_instances)
    }

    /**
        Creates a place document out of a `DataModel` instance.

        # Errors

        Errors if the instance is not a `DataModel`.
    */
    pub fn from_data_model_instance(i: Instance) -> DocumentResult<Self> {
        if i.get_class_name() != data_model::CLASS_NAME {
            return Err(DocumentError::FromDataModelInvalidArgs);
        }

        let mut dom = WeakDom::new(DomInstanceBuilder::new("ROOT"));
        let children: Vec<DomRef> = i
            .get_children()
            .iter()
            .map(|instance| instance.dom_ref)
            .collect();

        Instance::clone_multiple_into_external_dom(&children, &mut dom);
        postprocess_dom_for_place(&mut dom);

        Ok(Self {
            kind: DocumentKind::Place,
            format: DocumentFormat::default(),
            dom,
        })
    }

    /**
        Creates a model document out of an array of instances.

        # Errors

        Errors if any of the instances is a `DataModel`.
    */
    pub fn from_instance_array(v: Vec<Instance>) -> DocumentResult<Self> {
        for i in &v {
            if i.get_class_name() == data_model::CLASS_NAME {
                return Err(DocumentError::FromInstanceArrayInvalidArgs);
            }
        }

        let mut dom = WeakDom::new(DomInstanceBuilder::new("ROOT"));
        let instances: Vec<DomRef> = v.iter().map(|instance| instance.dom_ref).collect();

        Instance::clone_multiple_into_external_dom(&instances, &mut dom);
        postprocess_dom_for_model(&mut dom);

        Ok(Self {
            kind: DocumentKind::Model,
            format: DocumentFormat::default(),
            dom,
        })
    }
}
