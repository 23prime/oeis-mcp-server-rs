use std::collections::VecDeque;

use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use tracing::debug;

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

    pub async fn find_by_id(&self, id: &str) -> Result<Option<OEISSequence>, reqwest::Error> {
        let response = self
            .client
            .get(&self.url)
            .query(&[("fmt", "json"), ("q", &format!("id:{}", id))])
            .send()
            .await?;
        debug!("OEIS Response: {:?}", response);
        let oeis_response: Option<Vec<OEISSequence>> = response.json().await?;
        Ok(oeis_response.and_then(|sv| VecDeque::from(sv).pop_front()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::{Mock, MockServer};

    // helpers
    fn setup_test_client(server: &MockServer) -> OEISClient {
        OEISClient {
            url: format!("{}/search", server.base_url()),
            client: reqwest::Client::new(),
        }
    }

    fn mock_oeis_search<'a>(server: &'a MockServer, id: &str, status: u16, body: &str) -> Mock<'a> {
        server.mock(|when, then| {
            when.method(GET)
                .path("/search")
                .query_param("fmt", "json")
                .query_param("q", format!("id:{}", id));
            if status == 200 {
                then.status(status)
                    .header("Content-Type", "application/json")
                    .body(body);
            } else {
                then.status(status);
            }
        })
    }

    // tests
    #[tokio::test]
    async fn test_find_by_id() {
        let server = MockServer::start();
        let client = setup_test_client(&server);

        let _mock = mock_oeis_search(
            &server,
            "A000045",
            200,
            r#"
                [
                    {
                        "number": 45,
                        "data": "0, 1, 1, 2, 3, 5, 8, 13, 21, 34",
                        "name": "Fibonacci numbers",
                        "comment": ["The Fibonacci sequence is defined by the recurrence relation F(n) = F(n-1) + F(n-2) with seed values F(0)=0 and F(1)=1."],
                        "formula": ["F(n) = (phi^n - (1-phi)^n)/sqrt(5), where phi = (1 + sqrt(5))/2."],
                        "xref": ["A000045", "A001519"],
                        "keyword": "nonn"
                    }
                ]
                "#,
        );

        let result = client.find_by_id("A000045").await.unwrap();

        assert!(result.is_some());
        let found_sequence = result.unwrap();
        assert_eq!(found_sequence.number, 45);
        assert_eq!(found_sequence.name, "Fibonacci numbers");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let server = MockServer::start();
        let client = setup_test_client(&server);

        let _mock = mock_oeis_search(&server, "NON_EXISTENT", 200, "null");

        let result = client.find_by_id("NON_EXISTENT").await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_by_id_error() {
        let server = MockServer::start();
        let client = setup_test_client(&server);

        let _mock = mock_oeis_search(&server, "ERROR_CASE", 500, "");

        let result = client.find_by_id("ERROR_CASE").await;

        assert!(result.is_err());
    }
}
