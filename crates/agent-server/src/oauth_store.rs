use crate::cli::OAuthStoreArgs;
use agent_api::{oauth_router, oauth_router_with_store};
use agent_gateway::GatewayOAuthService;
use agent_store::{InMemoryOAuthStore, OAuthStore, SqliteOAuthStore};

pub(crate) fn oauth_service(
    args: OAuthStoreArgs,
) -> anyhow::Result<GatewayOAuthService<Box<dyn OAuthStore + Send>>> {
    match (args.oauth_store, args.oauth_key_hex) {
        (None, None) => Ok(GatewayOAuthService::new(
            Box::new(InMemoryOAuthStore::default()) as Box<dyn OAuthStore + Send>,
        )),
        (Some(path), Some(key_hex)) => Ok(GatewayOAuthService::new(
            Box::new(SqliteOAuthStore::open(path, parse_oauth_key(&key_hex)?)?)
                as Box<dyn OAuthStore + Send>,
        )),
        (Some(_), None) => anyhow::bail!("--oauth-key-hex is required with --oauth-store"),
        (None, Some(_)) => anyhow::bail!("--oauth-store is required with --oauth-key-hex"),
    }
}

pub(crate) fn oauth_router_for_store_args(
    oauth_store: Option<&std::path::PathBuf>,
    oauth_key_hex: Option<&String>,
) -> anyhow::Result<axum::Router> {
    match (oauth_store, oauth_key_hex) {
        (None, None) => Ok(oauth_router()),
        (Some(path), Some(key_hex)) => {
            let store = SqliteOAuthStore::open(path, parse_oauth_key(key_hex)?)?;
            Ok(oauth_router_with_store(store))
        }
        (Some(_), None) => anyhow::bail!("--oauth-key-hex is required with --oauth-store"),
        (None, Some(_)) => anyhow::bail!("--oauth-store is required with --oauth-key-hex"),
    }
}

pub(crate) fn parse_oauth_key(raw: &str) -> anyhow::Result<[u8; 32]> {
    let bytes = hex::decode(raw.trim())?;
    if bytes.len() != 32 {
        anyhow::bail!("OAuth store key must decode to 32 bytes");
    }
    let mut key = [0_u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

#[cfg(test)]
mod tests;
