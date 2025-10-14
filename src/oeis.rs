use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars::{self, JsonSchema},
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::oeis_client::{OEISClient, OEISSequence};

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
        let result = client
            .find_by_id(&id)
            .await
            .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))?;

        let result: OEISSequence = result.ok_or({
            McpError::new(
                ErrorCode::INVALID_PARAMS,
                format!("No sequence found (by id: {})", id),
                None,
            )
        })?;

        Ok(CallToolResult::structured(json!(FindResponse { result })))
    }
}

#[tool_handler]
impl ServerHandler for OEIS {
    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        info!("Listing resource templates");

        Ok(ListResourceTemplatesResult {
            resource_templates: vec![
                RawResourceTemplate {
                    uri_template: "oeis://sequence/{id}".to_string(),
                    name: "OEIS Sequence".to_string(),
                    title: None,
                    description: Some("OEIS sequence data by ID (e.g., A000045)".to_string()),
                    mime_type: Some("application/json".to_string()),
                }
                .no_annotation(),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        info!("Reading resource: {:?}", uri);

        // Parse URI pattern: oeis://sequence/{id}
        if let Some(id) = uri.strip_prefix("oeis://sequence/") {
            let client = OEISClient::new();
            let result = client
                .find_by_id(id)
                .await
                .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))?;

            let sequence: OEISSequence = result.ok_or_else(|| {
                McpError::new(
                    ErrorCode::INVALID_PARAMS,
                    format!("No sequence found (by id: {})", id),
                    None,
                )
            })?;

            // Return JSON representation of the sequence
            let json_content = serde_json::to_string_pretty(&sequence)
                .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))?;

            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(&json_content, uri)],
            })
        } else {
            Err(McpError::new(
                ErrorCode::INVALID_PARAMS,
                format!(
                    "Invalid resource URI: {}. Expected format: oeis://sequence/{{id}}",
                    uri
                ),
                Some(json!({"uri": uri})),
            ))
        }
    }
}
