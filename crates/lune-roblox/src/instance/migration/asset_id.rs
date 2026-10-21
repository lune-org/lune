use rbx_dom_weak::types::{Content, ContentType};

/**
    Gets the numeric asset id of a content, or `0` if
    it is not a uri, or not a uri with an asset id.
*/
pub(super) fn from_content(content: &Content) -> i64 {
    match content.value() {
        ContentType::Uri(uri) => parse_asset_id(uri).unwrap_or(0),
        _ => 0,
    }
}

/**
    Parses the numeric asset id out of a Roblox asset uri, such as
    `rbxassetid://1234` or `https://www.roblox.com/asset/?id=1234`.

    ### See Also
    * [Asset URIs](https://create.roblox.com/docs/projects/assets#asset-uris)
      on the Roblox Developer Hub
*/
fn parse_asset_id(uri: impl AsRef<str>) -> Option<i64> {
    let uri = uri.as_ref().trim().to_ascii_lowercase();

    if let Some(id) = uri.strip_prefix("rbxassetid://") {
        return parse_id(id);
    }

    // NOTE: rbxhttp is shorthand for ContentProvider.BaseUrl, which is www.roblox.com
    let web_path = if let Some(rest) = uri.strip_prefix("rbxhttp://") {
        Some(format!("roblox.com/{rest}"))
    } else {
        uri.strip_prefix("https://")
            .or_else(|| uri.strip_prefix("http://"))
            .map(|rest| rest.strip_prefix("www.").unwrap_or(rest).to_string())
    };

    if let Some(rest) = web_path {
        let (path, query) = rest.split_once('?')?;
        // NOTE: assetdelivery is not documented as an asset uri, but it is
        // the api that serves assets, and urls for it are common in the wild
        return match path.trim_end_matches('/') {
            "roblox.com/asset" | "assetdelivery.roblox.com/v1/asset" => query
                .split('&')
                .find_map(|param| param.strip_prefix("id="))
                .and_then(parse_id),
            _ => None,
        };
    }

    parse_id(&uri)
}

fn parse_id(id: &str) -> Option<i64> {
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    id.parse().ok().filter(|id| *id > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_asset_id_valid() {
        assert_eq!(parse_asset_id("1234"), Some(1234));
        assert_eq!(parse_asset_id("rbxassetid://1234"), Some(1234));
        assert_eq!(parse_asset_id("RBXASSETID://1234"), Some(1234));
        assert_eq!(parse_asset_id(" rbxassetid://1234 "), Some(1234));
        assert_eq!(parse_asset_id("rbxhttp://asset/?id=1234"), Some(1234));
        assert_eq!(parse_asset_id("rbxhttp://asset?id=1234"), Some(1234));
        assert_eq!(
            parse_asset_id("http://www.roblox.com/asset/?id=1234"),
            Some(1234)
        );
        assert_eq!(
            parse_asset_id("https://www.roblox.com/asset/?id=1234"),
            Some(1234)
        );
        assert_eq!(
            parse_asset_id("https://roblox.com/asset/?id=1234"),
            Some(1234)
        );
        assert_eq!(
            parse_asset_id("https://www.roblox.com/asset?id=1234"),
            Some(1234)
        );
        assert_eq!(
            parse_asset_id("https://www.roblox.com/asset/?id=1234&version=2"),
            Some(1234)
        );
        assert_eq!(
            parse_asset_id("https://www.roblox.com/asset/?version=2&id=1234"),
            Some(1234)
        );
        assert_eq!(
            parse_asset_id("https://assetdelivery.roblox.com/v1/asset/?id=1234"),
            Some(1234)
        );
        assert_eq!(
            parse_asset_id("https://assetdelivery.roblox.com/v1/asset?id=1234"),
            Some(1234)
        );
    }

    #[test]
    fn parse_asset_id_invalid() {
        assert_eq!(parse_asset_id(""), None);
        assert_eq!(parse_asset_id("rbxassetid://"), None);
        assert_eq!(parse_asset_id("rbxassetid://0"), None);
        assert_eq!(parse_asset_id("rbxassetid://abc"), None);
        assert_eq!(parse_asset_id("rbxassetid://12a"), None);
        assert_eq!(parse_asset_id("rbxgameasset://Images/Potion"), None);
        assert_eq!(parse_asset_id("rbxtemp://1"), None);
        assert_eq!(
            parse_asset_id("rbxhttp://Thumbs/Avatar.ashx?x=100&y=100&format=png"),
            None
        );
        assert_eq!(
            parse_asset_id("rbxasset://textures/ui/Scroll/scroll-top.png"),
            None
        );
        assert_eq!(
            parse_asset_id("rbxthumb://type=Asset&id=1234&w=150&h=150"),
            None
        );
        assert_eq!(
            parse_asset_id("https://www.roblox.com/library/1234/Name"),
            None
        );
        assert_eq!(parse_asset_id("https://example.com/asset/?id=1234"), None);
        assert_eq!(parse_asset_id("https://www.roblox.com/asset/?id="), None);
        assert_eq!(parse_asset_id("https://www.roblox.com/asset/"), None);
    }
}
