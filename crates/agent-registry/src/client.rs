use std::time::Duration;

use url::Url;

use crate::{McpRegistryResponse, NormalizedMcpServer, RegistryError, RegistryResult};

pub const DEFAULT_MCP_REGISTRY_BASE_URL: &str = "https://registry.modelcontextprotocol.io";

#[derive(Clone)]
pub struct McpRegistryClient {
    client: reqwest::Client,
    base_url: Url,
}

impl McpRegistryClient {
    pub fn new(base_url: Url) -> RegistryResult<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("agentcast/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| RegistryError::Request(error.to_string()))?;
        Ok(Self { client, base_url })
    }

    pub fn official() -> RegistryResult<Self> {
        let base_url = Url::parse(DEFAULT_MCP_REGISTRY_BASE_URL)
            .map_err(|error| RegistryError::InvalidInput(error.to_string()))?;
        Self::new(base_url)
    }

    pub async fn list_servers(
        &self,
        search: Option<&str>,
        limit: Option<usize>,
        cursor: Option<&str>,
    ) -> RegistryResult<Vec<NormalizedMcpServer>> {
        let response = self.list_servers_page(search, limit, cursor).await?;
        response.normalize()
    }

    pub async fn list_all_servers(
        &self,
        search: Option<&str>,
        page_limit: Option<usize>,
    ) -> RegistryResult<Vec<NormalizedMcpServer>> {
        let mut cursor = None::<String>;
        let mut servers = Vec::new();

        loop {
            let page = self
                .list_servers_page(search, page_limit, cursor.as_deref())
                .await?;
            cursor = page.next_cursor().map(ToOwned::to_owned);
            servers.extend(page.normalize()?);
            if cursor.is_none() {
                return Ok(servers);
            }
        }
    }

    pub async fn list_servers_page(
        &self,
        search: Option<&str>,
        limit: Option<usize>,
        cursor: Option<&str>,
    ) -> RegistryResult<McpRegistryResponse> {
        let mut url = self
            .base_url
            .join("/v0.1/servers")
            .map_err(|error| RegistryError::Request(format!("invalid registry URL: {error}")))?;
        {
            let mut query = url.query_pairs_mut();
            if let Some(search) = search.filter(|value| !value.trim().is_empty()) {
                query.append_pair("search", search);
            }
            if let Some(limit) = limit {
                query.append_pair("limit", &limit.to_string());
            }
            if let Some(cursor) = cursor.filter(|value| !value.trim().is_empty()) {
                query.append_pair("cursor", cursor);
            }
        }

        self.client
            .get(url)
            .send()
            .await
            .map_err(|error| RegistryError::Request(error.to_string()))?
            .error_for_status()
            .map_err(|error| RegistryError::Request(error.to_string()))?
            .json::<McpRegistryResponse>()
            .await
            .map_err(|error| RegistryError::InvalidData(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    #[test]
    fn official_client_uses_registry_base_url() {
        let client = McpRegistryClient::official().unwrap();
        assert_eq!(
            client.base_url.as_str().trim_end_matches('/'),
            DEFAULT_MCP_REGISTRY_BASE_URL
        );
    }

    #[tokio::test]
    async fn list_all_servers_follows_registry_pagination() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let base_url = Url::parse(&format!("http://{}", listener.local_addr().expect("addr")))
            .expect("base url");
        let handle = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept");
                let mut request = [0_u8; 2048];
                let read = stream.read(&mut request).expect("read");
                let request = String::from_utf8_lossy(&request[..read]);
                let body = if request.contains("cursor=next") {
                    r#"{"servers":[{"name":"two","version":"1"}],"metadata":{}}"#
                } else {
                    r#"{"servers":[{"name":"one","version":"1"}],"metadata":{"next_cursor":"next"}}"#
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).expect("write");
            }
        });

        let servers = McpRegistryClient::new(base_url)
            .expect("client")
            .list_all_servers(None, Some(1))
            .await
            .expect("servers");

        handle.join().expect("server thread");
        assert_eq!(
            servers
                .iter()
                .map(|server| server.name.as_str())
                .collect::<Vec<_>>(),
            ["one", "two"]
        );
    }
}
