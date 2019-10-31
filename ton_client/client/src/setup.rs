use client::ClientContext;
use dispatch::DispatchTable;
use types::ApiResult;
#[cfg(feature = "node_interaction")]
use types::ApiError;

#[cfg(feature = "node_interaction")]
use ton_sdk::{NodeClientConfig, RequestsConfig, QueriesConfig};

pub(crate) fn register(handlers: &mut DispatchTable) {
    #[cfg(feature = "node_interaction")]
    handlers.call_no_args("uninit", |_| Ok(ton_sdk::uninit()));
    #[cfg(not(feature = "node_interaction"))]
    handlers.call_no_args("uninit", |_| Ok(()));

    handlers.call("setup", setup);
    handlers.call_no_args("version", |_|Ok(env!("CARGO_PKG_VERSION")));
}


#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct SetupParams {
    pub default_workchain: Option<i32>,
    pub base_url: Option<String>,
    pub requests_url: Option<String>,
    pub queries_url: Option<String>,
    pub subscriptions_url: Option<String>,
}

#[cfg(feature = "node_interaction")]
fn setup(_context: &mut ClientContext, config: SetupParams) -> ApiResult<()> {
    fn replace_prefix(s: &String, prefix: &str, new_prefix: &str) -> String {
        format!("{}{}", new_prefix, s[prefix.len()..].to_string())
    }

    fn resolve_url(configured: Option<&String>, default: &str) -> String {
        let url = configured.unwrap_or(&default.to_string()).to_lowercase().trim().to_string();
        if url.starts_with("http://") ||
            url.starts_with("https://") ||
            url.starts_with("ws://") ||
            url.starts_with("wss://")
        {
            url
        } else {
            format!("https://{}", url)
        }
    }

    let base_url = resolve_url(
        config.base_url.as_ref(),
        "services.tonlabs.io",
    );

    let requests_url = resolve_url(
        config.requests_url.as_ref(),
        &format!("{}/topics/requests", base_url),
    );


    let queries_url = resolve_url(
        config.queries_url.as_ref(),
        &format!("{}/graphql", base_url),
    );


    let subscriptions_url = resolve_url(
        config.subscriptions_url.as_ref(),
        &if queries_url.starts_with("https://") {
            replace_prefix(&queries_url, "https://", "wss://")
        } else {
            replace_prefix(&queries_url, "http://", "ws://")
        }
    );

    let internal_config = NodeClientConfig {
        requests_config: RequestsConfig {
            requests_server: requests_url,
        },
        queries_config: QueriesConfig {
            queries_server: queries_url,
            subscriptions_server: subscriptions_url
        }
    };
    ton_sdk::init(Some(config.default_workchain.unwrap_or(0)), internal_config).map_err(|err|ApiError::config_init_failed(err))
}


#[cfg(not(feature = "node_interaction"))]
fn setup(_context: &mut ClientContext, config: SetupParams) -> ApiResult<()> {
    Ok(ton_sdk::Contract::set_default_workchain(Some(config.default_workchain.unwrap_or(0))))
}
