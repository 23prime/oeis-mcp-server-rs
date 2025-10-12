use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use tracing::debug;

type OEISResponse = Vec<OEISSequence>;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct OEISSequence {
    pub number: i64,
    pub data: String,
    pub name: String,
    pub comment: Vec<String>,
    pub formula: Vec<String>,
    pub xref: Vec<String>,
    pub keyword: String,
}

pub struct OEISClient {
    url: String,
    client: reqwest::Client,
}

impl OEISClient {
    pub fn new() -> Self {
        Self {
            url: "https://oeis.org/search".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn find_by_id(&self, id: &str) -> Result<OEISResponse, reqwest::Error> {
        let response = self
            .client
            .get(&self.url)
            .query(&[("fmt", "json"), ("q", &format!("id:{}", id))])
            .send()
            .await?;
        debug!("OEIS Response: {:?}", response);
        let oeis_response: OEISResponse = response.json().await?;
        Ok(oeis_response)
    }
}
