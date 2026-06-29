use std::path::PathBuf;

use clap::Args;
use moe_research_error::{Error, Result};

use crate::onboarding::config::{
    ConfigPlan, ProviderSelections, render_config, resolve_config_path, write_config_file,
};
use crate::onboarding::prompt::prompt_config_plan;

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct InitArgs {
    /// Path to the MoeResearch TOML configuration file.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Overwrite an existing configuration file.
    #[arg(long)]
    pub force: bool,
    /// Log the generated configuration without writing it.
    #[arg(long)]
    pub dry_run: bool,
    /// Skip guided prompts and use defaults plus provider flags.
    #[arg(long)]
    pub non_interactive: bool,
    /// Enable the `OpenAI` model provider in the generated config.
    #[arg(long)]
    pub enable_openai: bool,
    /// Enable the Grok search provider in the generated config.
    #[arg(long)]
    pub enable_grok: bool,
    /// Enable the Exa search provider in the generated config.
    #[arg(long)]
    pub enable_exa: bool,
    /// Enable the Tavily search provider in the generated config.
    #[arg(long)]
    pub enable_tavily: bool,
}

pub fn run(args: InitArgs) -> Result<()> {
    let selections = provider_selections(&args);
    let has_provider_flags = selections.any();
    let path = resolve_config_path(args.config);
    let mut plan = ConfigPlan::new(selections);

    if !args.dry_run && path.exists() && !args.force {
        return Err(Error::InvalidInput {
            message: format!(
                "{} already exists; pass --force to overwrite or choose --config",
                path.display()
            ),
        });
    }

    if !args.dry_run && !args.non_interactive && !has_provider_flags {
        plan = prompt_config_plan(plan)?;
    }

    let content = render_config(&plan);

    if args.dry_run {
        tracing::info!(config = %path.display(), content = %content, "would write MoeResearch config");
        log_next_steps(&path, &plan);
        return Ok(());
    }

    write_config_file(&path, &content)?;
    tracing::info!(config = %path.display(), "wrote MoeResearch config");
    log_next_steps(&path, &plan);
    Ok(())
}

fn provider_selections(args: &InitArgs) -> ProviderSelections {
    ProviderSelections {
        openai: args.enable_openai,
        grok: args.enable_grok,
        exa: args.enable_exa,
        tavily: args.enable_tavily,
    }
}

fn log_next_steps(path: &std::path::Path, plan: &ConfigPlan) {
    tracing::info!("secrets stay outside config; export the named api_key_env variables");
    if plan.openai.enabled {
        tracing::info!(env = %plan.openai.api_key_env, "export provider API key");
    }
    if plan.grok.enabled {
        tracing::info!(env = %plan.grok.api_key_env, "export provider API key");
    }
    if plan.exa.enabled {
        tracing::info!(env = %plan.exa.api_key_env, "export provider API key");
    }
    if plan.tavily.enabled {
        tracing::info!(env = %plan.tavily.api_key_env, "export provider API key");
    }
    if !plan.model_enabled() {
        tracing::info!("enable at least one model provider before running readiness checks");
    }
    if !plan.search_enabled() {
        tracing::info!(
            "search providers are optional, but search-enabled aspects need one enabled"
        );
    }
    tracing::info!(config = %path.display(), "next: moeresearch check --config <path>");
}
