use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info};

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct OEIS {
    tool_router: ToolRouter<OEIS>,
}

impl OEIS {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EmptyRequest {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindRequest {
    pub id: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct FindResponse {
    pub result: OEISSequence,
}

#[tool_router]
impl OEIS {
    #[tool(description = "Get a URL of OEIS entry.")]
    async fn get_url(&self, _: Parameters<EmptyRequest>) -> Result<CallToolResult, McpError> {
        let url = "https://oeis.org";
        Ok(CallToolResult::success(vec![Content::text(url)]))
    }

    #[tool(description = "Find a sequence by its ID.")]
    async fn find_by_id(
        &self,
        Parameters(FindRequest { id }): Parameters<FindRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("Find sequence by ID: {:?}", id);

        let client = OEISClient::new();
        let results = client
            .find_by_id(&id)
            .await
            .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))?;

        let result: OEISSequence = results.first().cloned().ok_or(McpError::new(
            ErrorCode::INVALID_PARAMS,
            format!("No sequence found (by id: {})", id),
            None,
        ))?;

        Ok(CallToolResult::structured(json!(FindResponse { result })))
    }
}

#[tool_handler]
impl ServerHandler for OEIS {}

// TODO: extract to `OEISClient`
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
