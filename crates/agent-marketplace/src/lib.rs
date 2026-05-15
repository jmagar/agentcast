mod conflict;
mod error;
mod mcp;
mod mcp_params;
mod plan;

pub use conflict::{InstallConflict, InstallConflictKind};
pub use error::{MarketplaceError, MarketplaceResult};
pub use mcp::{
    ApplyInstallPlanResult, InstallEnvResolution, apply_install_plan_to_config, install_env_merge,
    plan_mcp_server_install, resolve_install_env,
};
pub use mcp_params::{
    validate_env_name, validate_env_value, validate_registry_url, validate_runtime_hint,
    validate_stdio_argv,
};
pub use plan::{InstallPlan, InstallStep, InstallStepKind};
