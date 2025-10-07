use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
};

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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyRequest {}

#[tool_router]
impl OEIS {
    #[tool(description = "Get a URL of OEIS entry.")]
    async fn get_url(&self, _: Parameters<EmptyRequest>) -> Result<CallToolResult, McpError> {
        let url = "https://oeis.org";
        Ok(CallToolResult::success(vec![Content::text(url)]))
    }
}

#[tool_handler]
impl ServerHandler for OEIS {}
