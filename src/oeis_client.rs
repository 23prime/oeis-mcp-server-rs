use std::collections::VecDeque;

use async_trait::async_trait;
use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct OEISSequence {
    pub number: i64,
    pub data: String,
    pub name: String,
    pub comment: Option<Vec<String>>,
    pub formula: Option<Vec<String>>,
    pub xref: Option<Vec<String>>,
    pub keyword: String,
}

#[async_trait]
pub trait OEISClient: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<OEISSequence>>;
    async fn search_by_subsequence(&self, subsequence: &[i64])
    -> anyhow::Result<Vec<OEISSequence>>;
}

#[derive(Clone)]
pub struct OEISClientImpl {
    url: String,
    client: reqwest::Client,
}

impl OEISClientImpl {
    pub fn new() -> Self {
        Self {
            url: "https://oeis.org/search".to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl OEISClient for OEISClientImpl {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<OEISSequence>> {
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

    async fn search_by_subsequence(
        &self,
        subsequence: &[i64],
    ) -> anyhow::Result<Vec<OEISSequence>> {
        let subsequence_str = subsequence
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let response = self
            .client
            .get(&self.url)
            .query(&[("fmt", "json"), ("q", &format!("seq:{}", subsequence_str))])
            .send()
            .await?;
        debug!("OEIS Response: {:?}", response);
        let oeis_response: Option<Vec<OEISSequence>> = response.json().await?;
        Ok(oeis_response.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::{Mock, MockServer};

    // helpers
    fn setup_test_client(server: &MockServer) -> impl OEISClient {
        OEISClientImpl {
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

    fn mock_oeis_subsequence_search<'a>(
        server: &'a MockServer,
        subsequence: &str,
        status: u16,
        body: &str,
    ) -> Mock<'a> {
        server.mock(|when, then| {
            when.method(GET)
                .path("/search")
                .query_param("fmt", "json")
                .query_param("q", format!("seq:{}", subsequence));
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

    #[tokio::test]
    async fn test_search_by_subsequence_single_result() {
        let server = MockServer::start();
        let client = setup_test_client(&server);

        let _mock = mock_oeis_subsequence_search(
            &server,
            "1,1,2,3,5",
            200,
            r#"
                [
                    {
                        "number": 45,
                        "data": "0, 1, 1, 2, 3, 5, 8, 13, 21, 34",
                        "name": "Fibonacci numbers",
                        "comment": ["The Fibonacci sequence"],
                        "formula": ["F(n) = F(n-1) + F(n-2)"],
                        "xref": ["A000045"],
                        "keyword": "nonn"
                    }
                ]
                "#,
        );

        let result = client
            .search_by_subsequence(&[1, 1, 2, 3, 5])
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].number, 45);
        assert_eq!(result[0].name, "Fibonacci numbers");
    }

    #[tokio::test]
    async fn test_search_by_subsequence_multiple_results() {
        let server = MockServer::start();
        let client = setup_test_client(&server);

        let _mock = mock_oeis_subsequence_search(
            &server,
            "1,2,3",
            200,
            r#"
                [
                    {
                        "number": 27,
                        "data": "1, 2, 3, 4, 5",
                        "name": "Natural numbers",
                        "comment": ["The natural numbers"],
                        "formula": ["a(n) = n"],
                        "xref": [],
                        "keyword": "nonn"
                    },
                    {
                        "number": 290,
                        "data": "1, 2, 3, 5, 7",
                        "name": "Primes and composites",
                        "comment": ["Mixed sequence"],
                        "formula": [],
                        "xref": [],
                        "keyword": "nonn"
                    }
                ]
                "#,
        );

        let result = client.search_by_subsequence(&[1, 2, 3]).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].number, 27);
        assert_eq!(result[0].name, "Natural numbers");
        assert_eq!(result[1].number, 290);
        assert_eq!(result[1].name, "Primes and composites");
    }

    #[tokio::test]
    async fn test_search_by_subsequence_not_found() {
        let server = MockServer::start();
        let client = setup_test_client(&server);

        let _mock = mock_oeis_subsequence_search(&server, "999,888,777", 200, "null");

        let result = client
            .search_by_subsequence(&[999, 888, 777])
            .await
            .unwrap();

        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_search_by_subsequence_empty_result() {
        let server = MockServer::start();
        let client = setup_test_client(&server);

        let _mock = mock_oeis_subsequence_search(&server, "123,456,789", 200, "[]");

        let result = client
            .search_by_subsequence(&[123, 456, 789])
            .await
            .unwrap();

        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_search_by_subsequence_error() {
        let server = MockServer::start();
        let client = setup_test_client(&server);

        let _mock = mock_oeis_subsequence_search(&server, "1,2,3", 500, "");

        let result = client.search_by_subsequence(&[1, 2, 3]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_by_subsequence_negative_numbers() {
        let server = MockServer::start();
        let client = setup_test_client(&server);

        let _mock = mock_oeis_subsequence_search(
            &server,
            "-1,0,1",
            200,
            r#"
                [
                    {
                        "number": 12345,
                        "data": "-1, 0, 1, 0, -1",
                        "name": "Sequence with negative numbers",
                        "comment": [],
                        "formula": [],
                        "xref": [],
                        "keyword": "sign"
                    }
                ]
                "#,
        );

        let result = client.search_by_subsequence(&[-1, 0, 1]).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].number, 12345);
        assert_eq!(result[0].name, "Sequence with negative numbers");
    }

    #[tokio::test]
    async fn test_search_by_subsequence_empty_input() {
        let server = MockServer::start();
        let client = setup_test_client(&server);

        let _mock = mock_oeis_subsequence_search(&server, "", 200, "null");

        let result = client.search_by_subsequence(&[]).await.unwrap();

        assert_eq!(result.len(), 0);
    }
}
