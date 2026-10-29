use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsFormat {
    pub name: String,
    #[serde(default)]
    pub qty: Option<String>,
    #[serde(default)]
    pub descriptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsImage {
    pub uri: String,
    #[serde(default)]
    pub uri150: Option<String>,
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsRelease {
    pub id: u64,
    pub title: String,
    pub artists: Vec<DiscogsArtist>,
    pub year: Option<u16>,
    #[serde(default)]
    pub labels: Vec<DiscogsLabel>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub styles: Vec<String>,
    #[serde(default)]
    pub tracklist: Vec<DiscogsTrack>,
    #[serde(default)]
    pub credits: Vec<DiscogsCredit>,
    #[serde(rename = "extraartists", default)]
    pub extraartists: Vec<DiscogsArtist>,
    #[serde(rename = "master_id", default)]
    pub master_id: Option<u64>,
    #[serde(rename = "master_url", default)]
    pub master_url: Option<String>,
    #[serde(default)]
    pub format: Option<Vec<String>>,
    #[serde(default)]
    pub formats: Vec<DiscogsFormat>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub images: Vec<DiscogsImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsArtist {
    pub name: String,
    pub anv: Option<String>,
    pub join: Option<String>,
    pub role: Option<String>,
    pub tracks: Option<String>,
    pub id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsLabel {
    pub name: String,
    pub catno: Option<String>,
    pub entity_type: Option<String>,
    pub entity_type_name: Option<String>,
    pub id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsTrack {
    pub position: Option<String>,
    pub title: String,
    pub duration: Option<String>,
    pub type_: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsCredit {
    pub name: String,
    pub role: String,
    pub id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsSearchResult {
    pub id: u64,
    pub title: String,
    #[serde(deserialize_with = "deserialize_year", default)]
    pub year: Option<u16>,
    #[serde(default)]
    pub thumb: Option<String>,
    #[serde(rename = "cover_image", default)]
    pub cover_image: Option<String>,
    #[serde(rename = "resource_url", default)]
    pub resource_url: Option<String>,
    #[serde(rename = "master_id", default)]
    pub master_id: Option<u64>,
    #[serde(rename = "master_url", default)]
    pub master_url: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub format: Option<Vec<String>>,
    #[serde(default)]
    pub label: Option<Vec<String>>,
    #[serde(default)]
    pub catno: Option<String>,
    #[serde(default)]
    pub barcode: Vec<String>, // Always present, can be empty array
    #[serde(default)]
    pub community: Option<DiscogsCommunity>,
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub genre: Option<Vec<String>>,
    #[serde(default)]
    pub style: Option<Vec<String>>,
    #[serde(rename = "user_data", default)]
    pub user_data: Option<DiscogsUserData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsUserData {
    #[serde(rename = "in_wantlist")]
    pub in_wantlist: Option<bool>,
    #[serde(rename = "in_collection")]
    pub in_collection: Option<bool>,
}

// Helper to deserialize year which can be string or number
fn deserialize_year<'de, D>(deserializer: D) -> Result<Option<u16>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct YearVisitor;

    impl<'de> Visitor<'de> for YearVisitor {
        type Value = Option<u16>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or number representing a year, or null")
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<u16>, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                Ok(None)
            } else {
                value.parse().map(Some).map_err(de::Error::custom)
            }
        }

        fn visit_u64<E>(self, value: u64) -> Result<Option<u16>, E>
        where
            E: de::Error,
        {
            if value > u16::MAX as u64 {
                Err(de::Error::custom("year too large"))
            } else {
                Ok(Some(value as u16))
            }
        }

        fn visit_i64<E>(self, value: i64) -> Result<Option<u16>, E>
        where
            E: de::Error,
        {
            if value < 0 || value > u16::MAX as i64 {
                Err(de::Error::custom("year out of range"))
            } else {
                Ok(Some(value as u16))
            }
        }

        fn visit_none<E>(self) -> Result<Option<u16>, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Option<u16>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(YearVisitor)
        }

        fn visit_unit<E>(self) -> Result<Option<u16>, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_option(YearVisitor)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsCommunity {
    pub want: Option<u32>,
    pub have: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsSearchResponse {
    pub pagination: DiscogsPagination,
    pub results: Vec<DiscogsSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsPagination {
    pub page: u32,
    pub pages: u32,
    pub per_page: u32,
    pub items: u32,
    pub urls: Option<DiscogsPaginationUrls>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsPaginationUrls {
    pub next: Option<String>,
    pub prev: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsToken {
    pub oauth_token: String,
    pub oauth_token_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsAuthToken {
    pub oauth_token: String,
    pub oauth_token_secret: String,
}
