use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct DocsGlobal {
    pub documentation: String,
    pub keys: HashMap<String, String>,
    pub learn_more_link: String,
    pub code_sample: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct DocsFunctionParamLink {
    pub name: String,
    pub documentation: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct DocsFunction {
    #[serde(skip)]
    pub global_name: String,
    pub documentation: String,
    pub params: Vec<DocsFunctionParamLink>,
    pub returns: Vec<String>,
    pub learn_more_link: String,
    pub code_sample: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct DocsParam {
    #[serde(skip)]
    pub global_name: String,
    #[serde(skip)]
    pub function_name: String,
    pub documentation: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct DocsReturn {
    #[serde(skip)]
    pub global_name: String,
    #[serde(skip)]
    pub function_name: String,
    pub documentation: String,
}
