use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::item::DocItem;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DocsItemTag {
    Class(String),
    Within(String),
    Param((String, String)),
    Return(String),
    MustUse,
    ReadOnly,
    NewFields,
}

impl TryFrom<DocItem> for DocsItemTag {
    type Error = anyhow::Error;
    fn try_from(value: DocItem) -> Result<Self> {
        if let Some(name) = value.get_name() {
            Ok(match name.trim().to_ascii_lowercase().as_ref() {
                "class" => Self::Class(
                    value
                        .get_value()
                        .context("Missing class name for class tag")?
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
                "new_fields" => Self::NewFields,
                s => bail!("Unknown docs tag: '{}'", s),
            })
        } else {
            bail!("Doc item has no name")
        }
    }
}
