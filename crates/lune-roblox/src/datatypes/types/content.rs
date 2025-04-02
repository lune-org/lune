use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::{Content as DomContent, ContentType as DomContentType};

use lune_utils::TableBuilder;

use crate::{exports::LuaExportsTable, instance::Instance};

use super::{super::*, EnumItem};

/**
    An implementation of the [Content](https://create.roblox.com/docs/reference/engine/datatypes/Content) Roblox datatype.

    This implements all documented properties, methods & constructors of the Content type as of April 2025.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct Content(ContentType);

impl LuaExportsTable<'_> for Content {
    const EXPORT_NAME: &'static str = "Content";

    fn create_exports_table(lua: &'_ Lua) -> LuaResult<LuaTable<'_>> {
        let from_uri = |_, uri: String| Ok(Self(ContentType::Uri(uri)));

        let from_object = |_, obj: LuaUserDataRef<Instance>| Ok(Self(ContentType::Object(*obj)));

        TableBuilder::new(lua)?
            .with_value("none", Content(ContentType::None))?
            .with_function("fromUri", from_uri)?
            .with_function("fromObject", from_object)?
            .build_readonly()
    }
}

impl LuaUserData for Content {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("SourceType", |_, this| {
            let variant_name = match this.0 {
                ContentType::None => "None",
                ContentType::Uri(_) => "Uri",
                ContentType::Object(_) => "Object",
            };
            Ok(EnumItem::from_enum_name_and_name(
                "ContentSourceType",
                variant_name,
            ))
        });
        fields.add_field_method_get("Uri", |_, this| {
            if let ContentType::Uri(uri) = &this.0 {
                Ok(Some(uri.to_owned()))
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_get("Object", |_, this| {
            if let ContentType::Object(object) = &this.0 {
                Ok(Some(*object))
            } else {
                Ok(None)
            }
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Regardless of the actual content of the Content, Roblox just emits
        // `Content` when casting it to a string. We do not do that.
        write!(f, "Content(")?;
        match &self.0 {
            ContentType::None => write!(f, "None")?,
            ContentType::Uri(uri) => write!(f, "Uri={uri}")?,
            ContentType::Object(_) => write!(f, "Object")?,
        }
        write!(f, ")")
    }
}

impl TryFrom<DomContent> for Content {
    type Error = LuaError;

    fn try_from(value: DomContent) -> Result<Self, Self::Error> {
        // TODO: Replace with `DomContent.into_value()`.
        // rbx_types::Content is missing a method to get ownership of the
        // value right now so we have to do this.
        let converted_value = match value.value() {
            DomContentType::None => ContentType::None,
            DomContentType::Uri(uri) => ContentType::Uri(uri.to_owned()),
            DomContentType::Object(referent) => {
                if let Some(instance) = Instance::new_opt(*referent) {
                    ContentType::Object(instance)
                } else {
                    return Err(DomConversionError::FromDomValue {
                        from: "DomContentType",
                        to: "ContentType",
                        detail: Some(
                            "the value of DomContentType::Object must be a valid referent".into(),
                        ),
                    }
                    .into());
                }
            }
            _ => {
                return Err(DomConversionError::FromDomValue {
                    from: "???",
                    to: "ContentType",
                    detail: Some(format!(
                        "unknown variant of DomContentType (please open an issue at {})",
                        env!("CARGO_PKG_REPOSITORY")
                    )),
                }
                .into())
            }
        };

        Ok(Self(converted_value))
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ContentType {
    Uri(String),
    Object(Instance),
    None,
}

impl TryFrom<DomContentType> for ContentType {
    type Error = LuaError;

    fn try_from(value: DomContentType) -> Result<Self, Self::Error> {
        match value {
            DomContentType::None => Ok(Self::None),
            DomContentType::Uri(uri) => Ok(Self::Uri(uri)),
            DomContentType::Object(referent) => {
                if let Some(instance) = Instance::new_opt(referent) {
                    Ok(Self::Object(instance))
                } else {
                    Err(DomConversionError::FromDomValue {
                        from: "DomContentType",
                        to: "ContentType",
                        detail: Some(
                            "the value of DomContentType::Object must be a valid referent".into(),
                        ),
                    }
                    .into())
                }
            }
            _ => Err(DomConversionError::FromDomValue {
                from: "???",
                to: "ContentType",
                detail: Some(format!(
                    "unknown variant of DomContentType (please open an issue at {})",
                    env!("CARGO_PKG_REPOSITORY")
                )),
            }
            .into()),
        }
    }
}

impl From<ContentType> for DomContentType {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::None => Self::None,
            ContentType::Uri(uri) => Self::Uri(uri),
            ContentType::Object(object) => Self::Object(object.dom_ref),
        }
    }
}
