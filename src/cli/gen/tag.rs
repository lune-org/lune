use anyhow::{bail, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocsTagKind {
    Class,
    Within,
    Param,
    Return,
}

impl DocsTagKind {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_ref() {
            "class" => Ok(Self::Class),
            "within" => Ok(Self::Within),
            "param" => Ok(Self::Param),
            "return" => Ok(Self::Return),
            s => bail!("Unknown docs tag: '{}'", s),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DocsTag {
    pub kind: DocsTagKind,
    pub name: String,
    pub contents: String,
}

#[derive(Clone, Debug)]
pub struct DocsTagList {
    tags: Vec<DocsTag>,
}

impl DocsTagList {
    pub fn new() -> Self {
        Self { tags: vec![] }
    }

    pub fn push(&mut self, tag: DocsTag) {
        self.tags.push(tag);
    }

    pub fn contains(&mut self, kind: DocsTagKind) -> bool {
        self.tags.iter().any(|tag| tag.kind == kind)
    }

    pub fn find(&mut self, kind: DocsTagKind) -> Option<&DocsTag> {
        self.tags.iter().find(|tag| tag.kind == kind)
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
}

impl IntoIterator for DocsTagList {
    type Item = DocsTag;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.tags.into_iter()
    }
}
