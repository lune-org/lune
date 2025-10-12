use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::{Content as DomContent, ContentType};

use lune_utils::TableBuilder;

use crate::{exports::LuaExportsTable, instance::Instance};

use super::{super::*, EnumItem};

/**
    An implementation of the [Content](https://create.roblox.com/docs/reference/engine/datatypes/Content) Roblox datatype.

    This implements all documented properties, methods & constructors of the Content type as of April 2025.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct Content(ContentType);

impl LuaExportsTable for Content {
    const EXPORT_NAME: &'static str = "Content";

    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        let from_uri = |_: &Lua, uri: String| Ok(Self(ContentType::Uri(uri)));

        let from_object = |_: &Lua, obj: LuaUserDataRef<Instance>| {
            let database = rbx_reflection_database::get().unwrap();
            let instance_descriptor = database
                .classes
                .get("Instance")
                .expect("the reflection database should always have Instance in it");
            let param_descriptor = database.classes.get(obj.get_class_name()).expect(
                "you should not be able to construct an Instance that is not known to Lune",
            );
            if database.has_superclass(param_descriptor, instance_descriptor) {
                Err(LuaError::runtime(
                    "the provided object is a descendant class of 'Instance', expected one that was only an 'Object'",
                ))
            } else {
                Ok(Content(ContentType::Object(obj.dom_ref)))
            }
        };

        TableBuilder::new(lua)?
            .with_value("none", Content(ContentType::None))?
            .with_function("fromUri", from_uri)?
            .with_function("fromObject", from_object)?
            .build_readonly()
    }
}

impl LuaUserData for Content {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("SourceType", |_, this| {
            let variant_name = match &this.0 {
                ContentType::None => "None",
                ContentType::Uri(_) => "Uri",
                ContentType::Object(_) => "Object",
                other => {
                    return Err(LuaError::runtime(format!(
                        "cannot get SourceType: unknown ContentType variant '{other:?}'"
                    )));
                }
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
            if let ContentType::Object(referent) = &this.0 {
                Ok(Instance::new_opt(*referent))
            } else {
                Ok(None)
            }
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Regardless of the actual content of the Content, Roblox just emits
        // `Content` when casting it to a string. We do not do that.
        write!(f, "Content(")?;
        match &self.0 {
            ContentType::None => write!(f, "None")?,
            ContentType::Uri(uri) => write!(f, "Uri={uri}")?,
            ContentType::Object(_) => write!(f, "Object")?,
            other => write!(f, "UnknownType({other:?})")?,
        }
        write!(f, ")")
    }
}

impl From<DomContent> for Content {
    fn from(value: DomContent) -> Self {
        Self(value.value().clone())
    }
}

impl From<Content> for DomContent {
    fn from(value: Content) -> Self {
        match value.0 {
            ContentType::None => Self::none(),
            ContentType::Uri(uri) => Self::from_uri(uri),
            ContentType::Object(referent) => Self::from_referent(referent),
            other => unimplemented!("unknown variant of ContentType: {other:?}"),
        }
    }
}
