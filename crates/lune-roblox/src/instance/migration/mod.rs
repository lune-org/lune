/*!
    Support for old property names that Roblox has since migrated to new properties,
    such as `UICorner.CornerRadius` becoming four separate corner radius properties.

    Setting an old property performs the migration and stores the new properties.
    Getting an old property that is not stored reads the new property and reverses
    the migration, back to the type of the old property.
*/

use mlua::prelude::*;

use rbx_dom_weak::types::{ContentType, Enum, Variant as DomValue, VariantType as DomType};
use rbx_reflection::PropertyMigration;

use crate::{
    datatypes::types::{BrickColor, Color3},
    shared::instance::{PropertyInfo, find_property_info},
};

use super::Instance;

mod asset_id;

pub(crate) fn get(this: &Instance, info: &PropertyInfo) -> Option<DomValue> {
    let migration = info.migration?;
    let new_name = migration.new_property_names().first()?;
    let new_info = find_property_info(this.class_name, new_name)?;
    let new_value = this.get_property(new_name).or_else(|| {
        if let Some(default) = new_info.value_default {
            Some(default.clone())
        } else {
            new_info
                .enum_default
                .map(|value| Enum::from_u32(value).into())
        }
    })?;
    attempt_reverse_migration(info.value_type?, &new_info, new_value)
}

pub(crate) fn set(
    this: &Instance,
    prop_name: &str,
    migration: &PropertyMigration,
    value: DomValue,
) -> LuaResult<()> {
    let migrated = migration.perform(&value).map_err(|e| {
        LuaError::RuntimeError(format!("Failed to set property '{prop_name}' - {e}"))
    })?;
    for new_name in migration.new_property_names() {
        this.set_property(new_name, migrated.clone());
    }
    this.remove_property(prop_name);
    Ok(())
}

fn attempt_reverse_migration(
    old_type: DomType,
    new_info: &PropertyInfo,
    new_value: DomValue,
) -> Option<DomValue> {
    if new_value.ty() == old_type {
        return Some(new_value);
    }
    match (old_type, new_value) {
        // Colors
        (DomType::BrickColor, DomValue::Color3uint8(color)) => Some(DomValue::BrickColor(
            BrickColor::from(Color3::from(color)).into(),
        )),
        (DomType::BrickColor, DomValue::Color3(color)) => Some(DomValue::BrickColor(
            BrickColor::from(Color3::from(color)).into(),
        )),
        // Content
        (DomType::ContentId, DomValue::Content(content)) => {
            Some(DomValue::ContentId(match content.value() {
                ContentType::Uri(uri) => uri.as_str().into(),
                _ => "".into(),
            }))
        }
        (DomType::Int64, DomValue::Content(content)) => {
            Some(DomValue::Int64(asset_id::from_content(&content)))
        }
        // Small special cases
        (DomType::Bool, DomValue::Enum(insets))
            if new_info.enum_name.as_deref() == Some("ScreenInsets") =>
        {
            // NOTE: Only CoreUISafeInsets keeps the inset of the top bar,
            // every other value ignores it, just like IgnoreGuiInset = true
            let core_ui = enum_item_value("ScreenInsets", "CoreUISafeInsets")?;
            Some(DomValue::Bool(insets.to_u32() != core_ui))
        }
        _ => None,
    }
}

fn enum_item_value(enum_name: &str, item_name: &str) -> Option<u32> {
    let db = rbx_reflection_database::get().ok()?;
    db.enums.get(enum_name)?.items.get(item_name).copied()
}

#[cfg(test)]
mod tests {
    use rbx_dom_weak::types::{Content, UDim};
    use rbx_reflection::{PropertyKind, PropertySerialization};

    use super::*;

    fn migration(class: &str, prop: &str) -> &'static PropertyMigration<'static> {
        let db = rbx_reflection_database::get().unwrap();
        match &db.classes[class].properties[prop].kind {
            PropertyKind::Canonical {
                serialization: PropertySerialization::Migrate(migration),
            } => migration,
            _ => panic!("{class}.{prop} is not migrated"),
        }
    }

    fn new_info(class: &str, prop: &str) -> PropertyInfo {
        find_property_info(class, migration(class, prop).new_property_names()[0]).unwrap()
    }

    fn round_trip(class: &str, prop: &str, value: DomValue) -> Option<DomValue> {
        let migrated = migration(class, prop).perform(&value).unwrap();
        attempt_reverse_migration(value.ty(), &new_info(class, prop), migrated)
    }

    #[test]
    fn reverse_migrations_round_trip() {
        let corner = DomValue::UDim(UDim::new(0.5, 4));
        let content_id = DomValue::ContentId("rbxassetid://1234".into());
        let empty_content_id = DomValue::ContentId("".into());
        let red = DomValue::BrickColor(
            BrickColor::from(Color3::from(rbx_dom_weak::types::Color3::new(
                1.0, 0.0, 0.0,
            )))
            .into(),
        );

        assert_eq!(
            round_trip("UICorner", "CornerRadius", corner.clone()),
            Some(corner)
        );
        assert_eq!(round_trip("BasePart", "BrickColor", red.clone()), Some(red));
        assert_eq!(
            round_trip("Animation", "AnimationId", content_id.clone()),
            Some(content_id)
        );
        assert_eq!(
            round_trip("Animation", "AnimationId", empty_content_id.clone()),
            Some(empty_content_id)
        );
        assert_eq!(
            round_trip("CharacterMesh", "MeshId", DomValue::Int64(1234)),
            Some(DomValue::Int64(1234))
        );
        assert_eq!(
            round_trip("CharacterMesh", "MeshId", DomValue::Int64(0)),
            Some(DomValue::Int64(0))
        );
        assert_eq!(
            round_trip("ScreenGui", "IgnoreGuiInset", DomValue::Bool(true)),
            Some(DomValue::Bool(true))
        );
        assert_eq!(
            round_trip("ScreenGui", "IgnoreGuiInset", DomValue::Bool(false)),
            Some(DomValue::Bool(false))
        );
    }

    #[test]
    fn reverse_migrations_from_other_values() {
        let mesh = new_info("CharacterMesh", "MeshId");
        let animation = new_info("Animation", "AnimationId");
        let insets = new_info("ScreenGui", "IgnoreGuiInset");
        let none = enum_item_value("ScreenInsets", "None").unwrap();
        let topbar = enum_item_value("ScreenInsets", "TopbarSafeInsets").unwrap();

        assert_eq!(
            attempt_reverse_migration(DomType::Int64, &mesh, Content::none().into()),
            Some(DomValue::Int64(0))
        );
        assert_eq!(
            attempt_reverse_migration(DomType::ContentId, &animation, Content::none().into()),
            Some(DomValue::ContentId("".into()))
        );
        assert_eq!(
            attempt_reverse_migration(DomType::Bool, &insets, Enum::from_u32(none).into()),
            Some(DomValue::Bool(true))
        );
        assert_eq!(
            attempt_reverse_migration(DomType::Bool, &insets, Enum::from_u32(topbar).into()),
            Some(DomValue::Bool(true))
        );
    }

    #[test]
    fn reverse_migrations_check_the_enum() {
        let alignment = find_property_info("TextLabel", "TextXAlignment").unwrap();
        assert_eq!(
            attempt_reverse_migration(DomType::Bool, &alignment, Enum::from_u32(1).into()),
            None
        );
    }
}
