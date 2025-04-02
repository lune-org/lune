#![allow(clippy::items_after_statements)]

use mlua::prelude::*;

use rbx_dom_weak::{
    types::{Variant as DomValue, VariantType as DomType},
    Instance as DomInstance,
};

use crate::{
    datatypes::{
        attributes::{ensure_valid_attribute_name, ensure_valid_attribute_value},
        conversion::{DomValueToLua, LuaToDomValue},
        types::EnumItem,
        userdata_impl_eq, userdata_impl_to_string,
    },
    shared::instance::{class_is_a, find_property_info},
};

use super::{data_model, registry::InstanceRegistry, Instance};

#[allow(clippy::too_many_lines)]
pub fn add_methods<'lua, M: LuaUserDataMethods<'lua, Instance>>(m: &mut M) {
    m.add_meta_method(LuaMetaMethod::ToString, |lua, this, ()| {
        ensure_not_destroyed(this)?;
        userdata_impl_to_string(lua, this, ())
    });
    m.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
    m.add_meta_method(LuaMetaMethod::Index, instance_property_get);
    m.add_meta_method_mut(LuaMetaMethod::NewIndex, instance_property_set);
    m.add_method("Clone", |lua, this, ()| {
        ensure_not_destroyed(this)?;
        this.clone_instance().into_lua(lua)
    });
    m.add_method_mut("Destroy", |_, this, ()| {
        this.destroy();
        Ok(())
    });
    m.add_method_mut("ClearAllChildren", |_, this, ()| {
        this.clear_all_children();
        Ok(())
    });
    m.add_method("GetChildren", |lua, this, ()| {
        ensure_not_destroyed(this)?;
        this.get_children().into_lua(lua)
    });
    m.add_method("GetDescendants", |lua, this, ()| {
        ensure_not_destroyed(this)?;
        this.get_descendants().into_lua(lua)
    });
    m.add_method("GetFullName", |lua, this, ()| {
        ensure_not_destroyed(this)?;
        this.get_full_name().into_lua(lua)
    });
    m.add_method("GetDebugId", |lua, this, ()| {
        this.dom_ref.to_string().into_lua(lua)
    });
    m.add_method("FindFirstAncestor", |lua, this, name: String| {
        ensure_not_destroyed(this)?;
        this.find_ancestor(|child| child.name == name).into_lua(lua)
    });
    m.add_method(
        "FindFirstAncestorOfClass",
        |lua, this, class_name: String| {
            ensure_not_destroyed(this)?;
            this.find_ancestor(|child| child.class == class_name)
                .into_lua(lua)
        },
    );
    m.add_method(
        "FindFirstAncestorWhichIsA",
        |lua, this, class_name: String| {
            ensure_not_destroyed(this)?;
            this.find_ancestor(|child| class_is_a(child.class, &class_name).unwrap_or(false))
                .into_lua(lua)
        },
    );
    m.add_method(
        "FindFirstChild",
        |lua, this, (name, recursive): (String, Option<bool>)| {
            ensure_not_destroyed(this)?;
            let predicate = |child: &DomInstance| child.name == name;
            if matches!(recursive, Some(true)) {
                this.find_descendant(predicate).into_lua(lua)
            } else {
                this.find_child(predicate).into_lua(lua)
            }
        },
    );
    m.add_method(
        "FindFirstChildOfClass",
        |lua, this, (class_name, recursive): (String, Option<bool>)| {
            ensure_not_destroyed(this)?;
            let predicate = |child: &DomInstance| child.class == class_name;
            if matches!(recursive, Some(true)) {
                this.find_descendant(predicate).into_lua(lua)
            } else {
                this.find_child(predicate).into_lua(lua)
            }
        },
    );
    m.add_method(
        "FindFirstChildWhichIsA",
        |lua, this, (class_name, recursive): (String, Option<bool>)| {
            ensure_not_destroyed(this)?;
            let predicate =
                |child: &DomInstance| class_is_a(child.class, &class_name).unwrap_or(false);
            if matches!(recursive, Some(true)) {
                this.find_descendant(predicate).into_lua(lua)
            } else {
                this.find_child(predicate).into_lua(lua)
            }
        },
    );
    m.add_method("IsA", |_, this, class_name: String| {
        Ok(class_is_a(this.class_name, class_name).unwrap_or(false))
    });
    m.add_method(
        "IsAncestorOf",
        |_, this, instance: LuaUserDataRef<Instance>| {
            ensure_not_destroyed(this)?;
            Ok(instance
                .find_ancestor(|ancestor| ancestor.referent() == this.dom_ref)
                .is_some())
        },
    );
    m.add_method(
        "IsDescendantOf",
        |_, this, instance: LuaUserDataRef<Instance>| {
            ensure_not_destroyed(this)?;
            Ok(this
                .find_ancestor(|ancestor| ancestor.referent() == instance.dom_ref)
                .is_some())
        },
    );
    m.add_method("GetAttribute", |lua, this, name: String| {
        ensure_not_destroyed(this)?;
        match this.get_attribute(name) {
            Some(attribute) => Ok(LuaValue::dom_value_to_lua(lua, &attribute)?),
            None => Ok(LuaValue::Nil),
        }
    });
    m.add_method("GetAttributes", |lua, this, ()| {
        ensure_not_destroyed(this)?;
        let attributes = this.get_attributes();
        let tab = lua.create_table_with_capacity(0, attributes.len())?;
        for (key, value) in attributes {
            tab.set(key, LuaValue::dom_value_to_lua(lua, &value)?)?;
        }
        Ok(tab)
    });
    m.add_method(
        "SetAttribute",
        |lua, this, (attribute_name, lua_value): (String, LuaValue)| {
            ensure_not_destroyed(this)?;
            ensure_valid_attribute_name(&attribute_name)?;
            if lua_value.is_nil() || lua_value.is_null() {
                this.remove_attribute(attribute_name);
                Ok(())
            } else {
                match lua_value.lua_to_dom_value(lua, None) {
                    Ok(dom_value) => {
                        ensure_valid_attribute_value(&dom_value)?;
                        this.set_attribute(attribute_name, dom_value);
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            }
        },
    );
    m.add_method("GetTags", |_, this, ()| {
        ensure_not_destroyed(this)?;
        Ok(this.get_tags())
    });
    m.add_method("HasTag", |_, this, tag: String| {
        ensure_not_destroyed(this)?;
        Ok(this.has_tag(tag))
    });
    m.add_method("AddTag", |_, this, tag: String| {
        ensure_not_destroyed(this)?;
        this.add_tag(tag);
        Ok(())
    });
    m.add_method("RemoveTag", |_, this, tag: String| {
        ensure_not_destroyed(this)?;
        this.remove_tag(tag);
        Ok(())
    });
}

fn ensure_not_destroyed(inst: &Instance) -> LuaResult<()> {
    if inst.is_destroyed() {
        Err(LuaError::RuntimeError(
            "Instance has been destroyed".to_string(),
        ))
    } else {
        Ok(())
    }
}

/*
    Gets a property value for an instance.

    Getting a value does the following:

    1. Check if it is a special property like "ClassName", "Name" or "Parent"
    2. Check if a property exists for the wanted name
        2a. Get an existing instance property OR
        2b. Get a property from a known default value
    3. Get a current child of the instance
    4. No valid property or instance found, throw error
*/
fn instance_property_get<'lua>(
    lua: &'lua Lua,
    this: &Instance,
    prop_name: String,
) -> LuaResult<LuaValue<'lua>> {
    match prop_name.as_str() {
        "ClassName" => return this.get_class_name().into_lua(lua),
        "Parent" => {
            return this.get_parent().into_lua(lua);
        }
        _ => {}
    }

    ensure_not_destroyed(this)?;

    if prop_name.as_str() == "Name" {
        return this.get_name().into_lua(lua);
    }

    if let Some(info) = find_property_info(this.class_name, &prop_name) {
        if let Some(prop) = this.get_property(&prop_name) {
            if let DomValue::Enum(enum_value) = prop {
                let enum_name = info.enum_name.ok_or_else(|| {
                    LuaError::RuntimeError(format!(
                        "Failed to get property '{prop_name}' - encountered unknown enum",
                    ))
                })?;
                EnumItem::from_enum_name_and_value(&enum_name, enum_value.to_u32())
                    .ok_or_else(|| {
                        LuaError::RuntimeError(format!(
                            "Failed to get property '{}' - Enum.{} does not contain numeric value {}",
                            prop_name, enum_name, enum_value.to_u32()
                        ))
                    })?
                    .into_lua(lua)
            } else {
                Ok(LuaValue::dom_value_to_lua(lua, &prop)?)
            }
        } else if let (Some(enum_name), Some(enum_value)) = (info.enum_name, info.enum_default) {
            EnumItem::from_enum_name_and_value(&enum_name, enum_value)
                .ok_or_else(|| {
                    LuaError::RuntimeError(format!(
                        "Failed to get property '{prop_name}' - Enum.{enum_name} does not contain numeric value {enum_value}",
                    ))
                })?
                .into_lua(lua)
        } else if let Some(prop_default) = info.value_default {
            Ok(LuaValue::dom_value_to_lua(lua, prop_default)?)
        } else if info.value_type.is_some() {
            if info.value_type == Some(DomType::Ref) {
                Ok(LuaValue::Nil)
            } else {
                Err(LuaError::RuntimeError(format!(
                    "Failed to get property '{prop_name}' - missing default value",
                )))
            }
        } else {
            Err(LuaError::RuntimeError(format!(
                "Failed to get property '{prop_name}' - malformed property info",
            )))
        }
    } else if let Some(inst) = this.find_child(|inst| inst.name == prop_name) {
        Ok(LuaValue::UserData(lua.create_userdata(inst)?))
    } else if let Some(getter) = InstanceRegistry::find_property_getter(lua, this, &prop_name) {
        getter.call(*this)
    } else if let Some(method) = InstanceRegistry::find_method(lua, this, &prop_name) {
        Ok(LuaValue::Function(method))
    } else {
        Err(LuaError::RuntimeError(format!(
            "{prop_name} is not a valid member of {this}",
        )))
    }
}

/*
    Sets a property value for an instance.

    Setting a value does the following:

    1. Check if it is a special property like "ClassName", "Name" or "Parent"
    2. Check if a property exists for the wanted name
        2a. Set a strict enum from a given EnumItem OR
        2b. Set a normal property from a given value
*/
fn instance_property_set<'lua>(
    lua: &'lua Lua,
    this: &mut Instance,
    (prop_name, prop_value): (String, LuaValue<'lua>),
) -> LuaResult<()> {
    ensure_not_destroyed(this)?;

    match prop_name.as_str() {
        "ClassName" => {
            return Err(LuaError::RuntimeError(
                "Failed to set ClassName - property is read-only".to_string(),
            ));
        }
        "Name" => {
            let name = String::from_lua(prop_value, lua)?;
            this.set_name(name);
            return Ok(());
        }
        "Parent" => {
            if this.get_class_name() == data_model::CLASS_NAME {
                return Err(LuaError::RuntimeError(
                    "Failed to set Parent - DataModel can not be reparented".to_string(),
                ));
            }
            type Parent<'lua> = Option<LuaUserDataRef<'lua, Instance>>;
            let parent = Parent::from_lua(prop_value, lua)?;
            this.set_parent(parent.map(|p| *p));
            return Ok(());
        }
        _ => {}
    }

    if let Some(info) = find_property_info(this.class_name, &prop_name) {
        if let Some(enum_name) = info.enum_name {
            match LuaUserDataRef::<EnumItem>::from_lua(prop_value, lua) {
                Ok(given_enum) if given_enum.parent.desc.name == enum_name => {
                    this.set_property(prop_name, DomValue::Enum((*given_enum).clone().into()));
                    Ok(())
                }
                Ok(given_enum) => Err(LuaError::RuntimeError(format!(
                    "Failed to set property '{}' - expected Enum.{}, got Enum.{}",
                    prop_name, enum_name, given_enum.parent.desc.name
                ))),
                Err(e) => Err(e),
            }
        } else if let Some(dom_type) = info.value_type {
            match prop_value.lua_to_dom_value(lua, Some(dom_type)) {
                Ok(dom_value) => {
                    this.set_property(prop_name, dom_value);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        } else {
            Err(LuaError::RuntimeError(format!(
                "Failed to set property '{prop_name}' - malformed property info",
            )))
        }
    } else if let Some(setter) = InstanceRegistry::find_property_setter(lua, this, &prop_name) {
        setter.call((*this, prop_value))
    } else {
        Err(LuaError::RuntimeError(format!(
            "{prop_name} is not a valid member of {this}",
        )))
    }
}
