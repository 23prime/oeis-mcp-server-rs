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
pub struct OEIS<C: OEISClient> {
    client: C,
    tool_router: ToolRouter<OEIS<C>>,
}

impl<C: OEISClient + Clone + 'static> OEIS<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
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
impl<C: OEISClient + Clone + 'static> OEIS<C> {
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

        let result = self
            .client
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
impl<C: OEISClient + Clone + 'static> ServerHandler for OEIS<C> {
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
            let result = self
                .client
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock OEIS Client for testing
    struct MockOEISClient {
        responses: HashMap<String, Option<OEISSequence>>,
    }

    impl MockOEISClient {
        fn new() -> Self {
            Self {
                responses: HashMap::new(),
            }
        }

        fn with_sequence(mut self, id: &str, sequence: OEISSequence) -> Self {
            self.responses.insert(id.to_string(), Some(sequence));
            self
        }

        fn with_not_found(mut self, id: &str) -> Self {
            self.responses.insert(id.to_string(), None);
            self
        }
    }

    #[async_trait]
    impl OEISClient for MockOEISClient {
        async fn find_by_id(&self, id: &str) -> Result<Option<OEISSequence>, reqwest::Error> {
            Ok(self.responses.get(id).cloned().unwrap_or(None))
        }
    }

    fn create_test_sequence(number: i64, name: &str) -> OEISSequence {
        OEISSequence {
            number,
            data: "0, 1, 1, 2, 3, 5, 8".to_string(),
            name: name.to_string(),
            comment: vec!["Test comment".to_string()],
            formula: vec!["Test formula".to_string()],
            xref: vec!["A000001".to_string()],
            keyword: "nonn".to_string(),
        }
    }

    // test for mock client
    #[tokio::test]
    async fn test_mock_client() {
        // Test that our mock client works correctly
        let fibonacci = create_test_sequence(45, "Fibonacci numbers");
        let client = MockOEISClient::new().with_sequence("A000045", fibonacci.clone());

        let result = client.find_by_id("A000045").await.unwrap();
        assert!(result.is_some());
        let sequence = result.unwrap();
        assert_eq!(sequence.number, 45);
        assert_eq!(sequence.name, "Fibonacci numbers");
    }

    #[tokio::test]
    async fn test_mock_client_not_found() {
        let client = MockOEISClient::new().with_not_found("NON_EXISTENT");

        let result = client.find_by_id("NON_EXISTENT").await.unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_uri_parsing() {
        // Test URI parsing logic
        let uri = "oeis://sequence/A000045";
        let id = uri.strip_prefix("oeis://sequence/");
        assert_eq!(id, Some("A000045"));

        let invalid_uri = "invalid://uri";
        let id = invalid_uri.strip_prefix("oeis://sequence/");
        assert_eq!(id, None);
    }
}
