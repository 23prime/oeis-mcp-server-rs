use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::*,
    prompt, prompt_handler, prompt_router,
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
    prompt_router: PromptRouter<OEIS<C>>,
}

impl<C: OEISClient + Clone + 'static> OEIS<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    /// Find a sequence by ID from the OEIS API
    async fn find_sequence(&self, id: &str) -> Result<OEISSequence, McpError> {
        let result = self
            .client
            .find_by_id(id)
            .await
            .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))?;

        result.ok_or_else(|| {
            McpError::new(
                ErrorCode::INVALID_PARAMS,
                format!("No sequence found (by id: {})", id),
                None,
            )
        })
    }

    /// Search sequences by subsequence from the OEIS API
    async fn search_sequences(&self, subsequence: &[i64]) -> Result<Vec<OEISSequence>, McpError> {
        self.client
            .search_by_subsequence(subsequence)
            .await
            .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))
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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchRequest {
    pub subsequence: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchResponse {
    pub results: Vec<OEISSequence>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SequenceAnalysisRequest {
    /// The OEIS sequence ID to analyze (e.g., "A000045")
    pub sequence_id: String,
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

        let result = self.find_sequence(&id).await?;

        Ok(CallToolResult::structured(json!(FindResponse { result })))
    }

    #[tool(description = "Search sequences by subsequence.")]
    async fn search_by_subsequence(
        &self,
        Parameters(SearchRequest { subsequence }): Parameters<SearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("Search sequences by subsequence: {:?}", subsequence);

        let results = self.search_sequences(&subsequence).await?;

        Ok(CallToolResult::structured(json!(SearchResponse {
            results
        })))
    }
}

#[prompt_router]
impl<C: OEISClient + Clone + 'static> OEIS<C> {
    /// Provides a comprehensive analysis of an OEIS sequence
    #[prompt(
        description = "Analyzes an OEIS sequence in detail, providing mathematical context, patterns, and related sequences"
    )]
    async fn sequence_analysis(
        &self,
        Parameters(SequenceAnalysisRequest { sequence_id }): Parameters<SequenceAnalysisRequest>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        info!("Analyzing sequence: {:?}", sequence_id);
        let sequence = self.find_sequence(&sequence_id).await?;
        Ok(vec![
            self.build_user_message(&sequence_id),
            self.build_assistant_messages(&sequence),
        ])
    }

    fn build_user_message(&self, sequence_id: &str) -> PromptMessage {
        PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "Please provide a comprehensive analysis of OEIS sequence {}. \
                Include:\n\
                1. The definition and meaning of this sequence\n\
                2. Mathematical properties and patterns\n\
                3. Real-world applications or significance\n\
                4. Relationships to other sequences\n\
                5. Interesting facts or observations",
                sequence_id
            ),
        )
    }

    fn build_assistant_messages(&self, sequence: &OEISSequence) -> PromptMessage {
        let sequence_id_formatted = format!("A{:06}", sequence.number);
        let comments_section = self.empty_or_join("Comments", &sequence.comment);
        let formulas_section = self.empty_or_join("Formulas", &sequence.formula);
        let xref_section = self.empty_or_join("Cross-references", &sequence.xref);

        let analysis_context = format!(
            "# OEIS Sequence {}\n\n\
            **Name:** {}\n\n\
            **Data (first few terms):** {}\n\n\
            **Keywords:** {}\n\n\
            {}{}{}",
            sequence_id_formatted,
            sequence.name,
            sequence.data,
            sequence.keyword,
            comments_section,
            formulas_section,
            xref_section,
        );

        PromptMessage::new_text(PromptMessageRole::Assistant, analysis_context)
    }

    fn empty_or_join(&self, title: &str, contents: &Option<Vec<String>>) -> String {
        if contents.clone().is_none_or(|c| c.is_empty()) {
            String::new()
        } else {
            format!(
                "**{}:**\n{}\n\n",
                title,
                contents.as_ref().unwrap().join("\n")
            )
        }
    }
}

#[tool_handler]
#[prompt_handler]
impl<C: OEISClient + Clone + 'static> ServerHandler for OEIS<C> {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                title: Some("OEIS MCP server".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: Some(env!("CARGO_PKG_DESCRIPTION").to_string()),
                icons: None,
                website_url: Some("https://github.com/23prime/oeis-mcp-server-rs".to_string()),
            },
            instructions: Some("This server provides access to the OEIS (Online Encyclopedia of Integer Sequences) database. Tools: get_url (returns the OEIS homepage URL), find_by_id (search for a sequence by ID like 'A000045'), search_by_subsequence (search for sequences matching a given subsequence like [1,1,2,3,5]). Prompts: sequence_analysis (provides comprehensive analysis of an OEIS sequence). Resources: oeis://sequence/{id} (direct access to sequence data as JSON). Use this server to look up integer sequences, analyze their mathematical properties, and explore relationships between sequences.".to_string()),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
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
                    icons: None,
                }
                .no_annotation(),
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParams { uri, .. }: ReadResourceRequestParams,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        info!("Reading resource: {:?}", uri);

        // Parse URI pattern: oeis://sequence/{id}
        if let Some(id) = uri.strip_prefix("oeis://sequence/") {
            let sequence = self.find_sequence(id).await?;

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
    use anyhow::anyhow;
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock OEIS Client for testing
    #[derive(Clone)]
    enum MockResponse {
        // TODO: normalize Option / Vec
        Success(Option<OEISSequence>),
        SuccessMulti(Vec<OEISSequence>),
        Error,
    }

    #[derive(Clone)]
    struct MockOEISClient {
        responses: HashMap<String, MockResponse>,
    }

    impl MockOEISClient {
        fn new() -> Self {
            Self {
                responses: HashMap::new(),
            }
        }

        fn with_sequence(mut self, id: &str, sequence: OEISSequence) -> Self {
            self.responses
                .insert(id.to_string(), MockResponse::Success(Some(sequence)));
            self
        }

        fn with_sequences(mut self, subsequence: &[i64], sequences: Vec<OEISSequence>) -> Self {
            self.responses.insert(
                subsequence
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
                MockResponse::SuccessMulti(sequences),
            );
            self
        }

        fn with_not_found(mut self, id: &str) -> Self {
            self.responses
                .insert(id.to_string(), MockResponse::Success(None));
            self
        }

        fn with_error(mut self, id: &str) -> Self {
            self.responses.insert(id.to_string(), MockResponse::Error);
            self
        }
    }

    #[async_trait]
    impl OEISClient for MockOEISClient {
        async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<OEISSequence>> {
            match self.responses.get(id) {
                Some(MockResponse::Success(sequence)) => Ok(sequence.clone()),
                Some(MockResponse::SuccessMulti(_)) => {
                    Err(anyhow!("MockOEISClient: use Success for find_by_id"))
                }
                Some(MockResponse::Error) => Err(anyhow!("Mock error")),
                None => Ok(None),
            }
        }

        async fn search_by_subsequence(
            &self,
            subsequence: &[i64],
        ) -> anyhow::Result<Vec<OEISSequence>> {
            let key = subsequence
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",");

            match self.responses.get(&key) {
                Some(MockResponse::SuccessMulti(sequences)) => Ok(sequences.clone()),
                Some(MockResponse::Success(_)) => Err(anyhow!(
                    "MockOEISClient: use SuccessMulti for subsequence searches"
                )),
                Some(MockResponse::Error) => Err(anyhow!("Mock error")),
                None => Ok(vec![]),
            }
        }
    }

    fn create_test_sequence(number: i64, name: &str) -> OEISSequence {
        OEISSequence {
            number,
            data: "0, 1, 1, 2, 3, 5, 8".to_string(),
            name: name.to_string(),
            comment: Some(vec!["Test comment".to_string()]),
            formula: Some(vec!["Test formula".to_string()]),
            xref: Some(vec!["A000001".to_string()]),
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
        let uri = "oeis://sequence/A000045";
        let id = uri.strip_prefix("oeis://sequence/");
        assert_eq!(id, Some("A000045"));
    }

    #[test]
    fn test_uri_parsing_invalid() {
        let invalid_uri = "invalid://uri";
        let id = invalid_uri.strip_prefix("oeis://sequence/");
        assert_eq!(id, None);
    }

    // test for find_sequence helper
    #[tokio::test]
    async fn test_find_sequence_success() {
        let fibonacci = create_test_sequence(45, "Fibonacci numbers");
        let oeis = OEIS::new(MockOEISClient::new().with_sequence("A000045", fibonacci.clone()));

        let result = oeis.find_sequence("A000045").await;
        assert!(result.is_ok());

        let sequence = result.unwrap();
        assert_eq!(sequence.number, 45);
        assert_eq!(sequence.name, "Fibonacci numbers");
    }

    #[tokio::test]
    async fn test_find_sequence_not_found() {
        let oeis = OEIS::new(MockOEISClient::new().with_not_found("NON_EXISTENT"));

        let result = oeis.find_sequence("NON_EXISTENT").await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode::INVALID_PARAMS);
        assert!(error.message.contains("No sequence found"));
    }

    #[tokio::test]
    async fn test_find_sequence_error() {
        let oeis = OEIS::new(MockOEISClient::new().with_error("ERROR_CASE"));

        let result = oeis.find_sequence("ERROR_CASE").await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode::INTERNAL_ERROR);
    }

    // helpers for tool router definition checking
    fn get_tool(tools: &[Tool], name: &str) -> Option<Tool> {
        tools.iter().find(|t| t.name == name).cloned()
    }

    fn get_tool_description(tool: Tool) -> String {
        tool.description.as_ref().unwrap().clone().into_owned()
    }

    #[test]
    fn test_tool_router_definition() {
        let oeis = OEIS::new(MockOEISClient::new());

        let tools = oeis.tool_router.list_all();
        assert!(tools.len() == 3);

        let get_url_tool = get_tool(&tools, "get_url");
        assert!(get_url_tool.is_some());
        assert!(get_tool_description(get_url_tool.unwrap()) == "Get a URL of OEIS entry.");

        let find_by_id_tool = get_tool(&tools, "find_by_id");
        assert!(find_by_id_tool.is_some());
        assert!(get_tool_description(find_by_id_tool.unwrap()) == "Find a sequence by its ID.");

        let search_by_subsequence_tool = get_tool(&tools, "search_by_subsequence");
        assert!(search_by_subsequence_tool.is_some());
        assert!(
            get_tool_description(search_by_subsequence_tool.unwrap())
                == "Search sequences by subsequence."
        );
    }

    #[tokio::test]
    async fn test_get_url_tool() {
        let oeis = OEIS::new(MockOEISClient::new());

        let result = oeis.get_url(Parameters(EmptyRequest {})).await;
        assert!(result.is_ok());

        let content = result.unwrap().content;
        assert_eq!(content.len(), 1);
        assert_eq!(content.first().unwrap(), &Content::text("https://oeis.org"));
    }

    #[tokio::test]
    async fn test_find_by_id_tool_found() {
        let fibonacci = create_test_sequence(45, "Fibonacci numbers");
        let oeis = OEIS::new(MockOEISClient::new().with_sequence("A000045", fibonacci.clone()));
        let params = Parameters(FindRequest {
            id: "A000045".to_string(),
        });

        let result = oeis.find_by_id(params).await;
        assert!(result.is_ok());

        let content = result.unwrap().content;
        assert_eq!(content.len(), 1);

        assert_eq!(
            content.first().unwrap(),
            &Content::json(json!(FindResponse { result: fibonacci })).unwrap()
        );
    }

    #[tokio::test]
    async fn test_find_by_id_tool_not_found() {
        let oeis = OEIS::new(MockOEISClient::new().with_not_found("NON_EXISTENT"));
        let params = Parameters(FindRequest {
            id: "NON_EXISTENT".to_string(),
        });

        let result = oeis.find_by_id(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode::INVALID_PARAMS);
        assert!(error.message.contains("No sequence found"));
    }

    #[tokio::test]
    async fn test_find_by_id_tool_error() {
        let oeis = OEIS::new(MockOEISClient::new().with_error("ERROR_CASE"));
        let params = Parameters(FindRequest {
            id: "ERROR_CASE".to_string(),
        });

        let result = oeis.find_by_id(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode::INTERNAL_ERROR);
    }

    #[tokio::test]
    async fn test_search_by_subsequence_tool_found() {
        let fibonacci = create_test_sequence(45, "Fibonacci numbers");
        let catalan = create_test_sequence(108, "Catalan numbers");
        let sequences = vec![fibonacci.clone(), catalan.clone()];

        let oeis = OEIS::new(
            MockOEISClient::new().with_sequences(&[0, 1, 1, 2, 3, 5, 8], sequences.clone()),
        );
        let params = Parameters(SearchRequest {
            subsequence: vec![0, 1, 1, 2, 3, 5, 8],
        });

        let result = oeis.search_by_subsequence(params).await;
        assert!(result.is_ok());

        let content = result.unwrap().content;
        assert_eq!(content.len(), 1);

        assert_eq!(
            content.first().unwrap(),
            &Content::json(json!(SearchResponse { results: sequences })).unwrap()
        );
    }

    #[tokio::test]
    async fn test_search_by_subsequence_tool_not_found() {
        let oeis = OEIS::new(MockOEISClient::new());
        let params = Parameters(SearchRequest {
            subsequence: vec![999, 888, 777],
        });

        let result = oeis.search_by_subsequence(params).await;
        assert!(result.is_ok());

        let content = result.unwrap().content;
        assert_eq!(content.len(), 1);

        assert_eq!(
            content.first().unwrap(),
            &Content::json(json!(SearchResponse { results: vec![] })).unwrap()
        );
    }

    #[tokio::test]
    async fn test_search_by_subsequence_tool_error() {
        let subsequence = vec![1, 2, 3];
        let oeis = OEIS::new(
            MockOEISClient::new().with_error(
                &subsequence
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            ),
        );
        let params = Parameters(SearchRequest {
            subsequence: subsequence.clone(),
        });

        let result = oeis.search_by_subsequence(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode::INTERNAL_ERROR);
        assert!(error.message.contains("Mock error"));
    }

    // Test for prompts
    #[test]
    fn test_prompt_router_definition() {
        let oeis = OEIS::new(MockOEISClient::new());
        assert!(oeis.prompt_router.list_all().len() == 1);
    }

    #[tokio::test]
    async fn test_sequence_analysis_prompt() {
        let fibonacci = create_test_sequence(45, "Fibonacci numbers");
        let oeis = OEIS::new(MockOEISClient::new().with_sequence("A000045", fibonacci.clone()));

        let params = Parameters(SequenceAnalysisRequest {
            sequence_id: "A000045".to_string(),
        });

        let result = oeis.sequence_analysis(params).await;
        assert!(result.is_ok());

        let messages = result.unwrap();
        assert_eq!(messages.len(), 2);

        // Check first message is from user
        assert_eq!(messages[0].role, PromptMessageRole::User);
        if let PromptMessageContent::Text { text } = &messages[0].content {
            assert!(text.contains("comprehensive analysis"));
            assert!(text.contains("A000045"));
        } else {
            panic!("Expected text content");
        }

        // Check second message is from assistant with sequence data
        assert_eq!(messages[1].role, PromptMessageRole::Assistant);
        if let PromptMessageContent::Text { text } = &messages[1].content {
            assert!(text.contains("Fibonacci numbers"));
            assert!(text.contains("A000045"));
            assert!(text.contains("0, 1, 1, 2, 3, 5, 8"));
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_sequence_analysis_prompt_not_found() {
        let oeis = OEIS::new(MockOEISClient::new().with_not_found("NON_EXISTENT"));

        let params = Parameters(SequenceAnalysisRequest {
            sequence_id: "NON_EXISTENT".to_string(),
        });

        let result = oeis.sequence_analysis(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode::INVALID_PARAMS);
        assert!(error.message.contains("No sequence found"));
    }

    #[tokio::test]
    async fn test_sequence_analysis_prompt_error() {
        let oeis = OEIS::new(MockOEISClient::new().with_error("ERROR_CASE"));

        let params = Parameters(SequenceAnalysisRequest {
            sequence_id: "ERROR_CASE".to_string(),
        });

        let result = oeis.sequence_analysis(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode::INTERNAL_ERROR);
    }
}
