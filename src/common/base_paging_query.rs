use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BasePagingQuery {
    pub page: Option<u64>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u64>,
    pub search: Option<String>,
}
