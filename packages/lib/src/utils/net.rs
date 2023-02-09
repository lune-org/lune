use mlua::prelude::*;
use reqwest::{IntoUrl, Method, RequestBuilder};

#[derive(Clone)]
pub struct NetClient(reqwest::Client);

impl NetClient {
    pub fn new(client: reqwest::Client) -> Self {
        Self(client)
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        self.0.request(method, url)
    }
}

impl LuaUserData for NetClient {}

pub fn get_github_owner_and_repo() -> (String, String) {
    let (github_owner, github_repo) = env!("CARGO_PKG_REPOSITORY")
        .strip_prefix("https://github.com/")
        .unwrap()
        .split_once('/')
        .unwrap();
    (github_owner.to_owned(), github_repo.to_owned())
}

pub fn get_request_user_agent_header() -> String {
    let (github_owner, github_repo) = get_github_owner_and_repo();
    format!("{github_owner}-{github_repo}-cli")
}
