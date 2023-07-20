use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::item::DefinitionsItem;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DefinitionsItemTag {
    Class(String),
    Type(String),
    Within(String),
    Param((String, String)),
    Return(String),
    MustUse,
    ReadOnly,
    ReadWrite,
}

#[allow(dead_code)]
impl DefinitionsItemTag {
    pub fn is_class(&self) -> bool {
        matches!(self, Self::Class(_))
    }

    pub fn is_type(&self) -> bool {
        matches!(self, Self::Class(_))
    }

    pub fn is_within(&self) -> bool {
        matches!(self, Self::Within(_))
    }

    pub fn is_param(&self) -> bool {
        matches!(self, Self::Param(_))
    }

    pub fn is_return(&self) -> bool {
        matches!(self, Self::Return(_))
    }

    pub fn is_must_use(&self) -> bool {
        self == &Self::MustUse
    }

    pub fn is_read_only(&self) -> bool {
        self == &Self::ReadOnly
    }

    pub fn is_read_write(&self) -> bool {
        self == &Self::ReadWrite
    }
}

impl TryFrom<&DefinitionsItem> for DefinitionsItemTag {
    type Error = anyhow::Error;
    fn try_from(value: &DefinitionsItem) -> Result<Self> {
        if let Some(name) = value.get_name() {
            Ok(match name.trim().to_ascii_lowercase().as_ref() {
                "class" => Self::Class(
                    value
                        .get_value()
                        .context("Missing class name for class tag")?
                        .to_string(),
                ),
                "type" => Self::Class(
                    value
                        .get_value()
                        .context("Missing type name for type tag")?
                        .to_string(),
                ),
                "within" => Self::Within(
                    value
                        .get_value()
                        .context("Missing class name for within tag")?
                        .to_string(),
                ),
                "param" => Self::Param((
                    value
                        .get_meta()
                        .context("Missing param name for param tag")?
                        .to_string(),
                    value
                        .get_value()
                        .context("Missing param value for param tag")?
                        .to_string(),
                )),
                "return" => Self::Return(
                    value
                        .get_value()
                        .context("Missing description for return tag")?
                        .to_string(),
                ),
                "must_use" => Self::MustUse,
                "read_only" => Self::ReadOnly,
                "read_write" => Self::ReadWrite,
                s => bail!("Unknown docs tag: '{}'", s),
            })
        } else {
            bail!("Doc item has no name")
        }
    }
}
