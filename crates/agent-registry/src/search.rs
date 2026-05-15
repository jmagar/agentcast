use crate::NormalizedMcpServer;

#[cfg(test)]
mod tests;

pub fn search_servers(
    servers: &[NormalizedMcpServer],
    query: &str,
    limit: usize,
) -> Vec<NormalizedMcpServer> {
    if limit == 0 {
        return Vec::new();
    }
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return servers.iter().take(limit).cloned().collect();
    }

    servers
        .iter()
        .filter(|server| {
            server.name.to_ascii_lowercase().contains(&needle)
                || server
                    .title
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&needle)
                || server
                    .description
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&needle)
        })
        .take(limit)
        .cloned()
        .collect()
}
