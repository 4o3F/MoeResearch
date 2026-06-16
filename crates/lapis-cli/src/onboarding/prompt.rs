use inquire::{Confirm, Text};
use lapis_error::{Error, Result};

use crate::onboarding::config::{ConfigPlan, ProviderPlan};

pub fn prompt_config_plan(mut plan: ConfigPlan) -> Result<ConfigPlan> {
    plan.openai.enabled = prompt_bool("Enable OpenAI model provider", plan.openai.enabled)?;
    if plan.openai.enabled {
        prompt_provider("OpenAI", &mut plan.openai, true)?;
    }

    plan.grok.enabled = prompt_bool("Enable Grok search provider", plan.grok.enabled)?;
    if plan.grok.enabled {
        prompt_provider("Grok", &mut plan.grok, true)?;
    }

    plan.exa.enabled = prompt_bool("Enable Exa search provider", plan.exa.enabled)?;
    if plan.exa.enabled {
        prompt_provider("Exa", &mut plan.exa, false)?;
    }

    plan.tavily.enabled = prompt_bool("Enable Tavily search provider", plan.tavily.enabled)?;
    if plan.tavily.enabled {
        prompt_provider("Tavily", &mut plan.tavily, false)?;
    }

    Ok(plan)
}

fn prompt_provider(name: &str, provider: &mut ProviderPlan, takes_model: bool) -> Result<()> {
    provider.api_key_env = prompt_text(
        &format!("{name} API key environment variable"),
        &provider.api_key_env,
    )?;
    provider.base_url = prompt_text(&format!("{name} base URL"), &provider.base_url)?;

    if takes_model {
        let default_model = provider.model.as_deref().unwrap_or_default();
        provider.model = Some(prompt_text(&format!("{name} model"), default_model)?);
    }

    Ok(())
}

fn prompt_bool(prompt: &str, default: bool) -> Result<bool> {
    Confirm::new(prompt)
        .with_default(default)
        .prompt()
        .map_err(|source| prompt_error(&source))
}

fn prompt_text(prompt: &str, default: &str) -> Result<String> {
    Text::new(prompt)
        .with_default(default)
        .prompt()
        .map(|answer| {
            let answer = answer.trim();
            if answer.is_empty() {
                default.to_owned()
            } else {
                answer.to_owned()
            }
        })
        .map_err(|source| prompt_error(&source))
}

fn prompt_error(source: &inquire::InquireError) -> Error {
    Error::Internal {
        message: format!("failed to read onboarding prompt input: {source}"),
    }
}
