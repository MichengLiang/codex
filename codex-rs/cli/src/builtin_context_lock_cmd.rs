use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use codex_config::LoaderOverrides;
use codex_config::builtin_context_lock::generate_builtin_context_lock;
use codex_config::config_toml::ConfigToml;
use codex_core::config::find_codex_home;
use codex_core::config::load_config_as_toml_with_cli_and_loader_overrides;
use codex_features::Feature;
use codex_features::FeatureConfigSource;
use codex_features::FeatureOverrides;
use codex_features::Features;
use codex_models_manager::ModelsManagerConfig;
use codex_models_manager::bundled_models_response;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelPreset;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_cli::CliConfigOverrides;
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(bin_name = "codex builtin-context-lock")]
pub(crate) struct BuiltinContextLockCli {
    #[clap(flatten)]
    pub config_overrides: CliConfigOverrides,

    #[command(subcommand)]
    pub subcommand: BuiltinContextLockSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum BuiltinContextLockSubcommand {
    /// Generate a builtin context lock JSON file.
    Generate(BuiltinContextLockGenerateCommand),
}

#[derive(Debug, clap::Parser)]
pub(crate) struct BuiltinContextLockGenerateCommand {
    /// Path to write the generated builtin context lock JSON.
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,
}

pub(crate) async fn run(
    cli: BuiltinContextLockCli,
    root_config_overrides: CliConfigOverrides,
) -> Result<()> {
    let BuiltinContextLockCli {
        config_overrides,
        subcommand,
    } = cli;
    let config_overrides = merged_config_overrides(root_config_overrides, config_overrides);

    match subcommand {
        BuiltinContextLockSubcommand::Generate(command) => {
            run_generate(command, config_overrides).await
        }
    }
}

async fn run_generate(
    command: BuiltinContextLockGenerateCommand,
    config_overrides: CliConfigOverrides,
) -> Result<()> {
    let codex_home = find_codex_home()?;
    let cwd = AbsolutePathBuf::current_dir()?;
    let config_toml = load_config_as_toml_with_cli_and_loader_overrides(
        &codex_home,
        Some(&cwd),
        config_overrides
            .parse_overrides()
            .map_err(anyhow::Error::msg)?,
        LoaderOverrides::default(),
    )
    .await?;

    let output_path = resolve_output_path(command.output, &config_toml)?;
    let lock = generate_builtin_context_lock(model_catalog_base_instructions(&config_toml)?);
    let json = serde_json::to_string_pretty(&lock).context("serialize builtin context lock")?;
    if let Some(parent) = output_path.as_path().parent() {
        tokio::fs::create_dir_all(parent).await.with_context(|| {
            format!(
                "failed to create parent directory for {}",
                output_path.as_path().display()
            )
        })?;
    }
    tokio::fs::write(output_path.as_path(), format!("{json}\n"))
        .await
        .with_context(|| {
            format!(
                "failed to write builtin context lock to {}",
                output_path.as_path().display()
            )
        })?;
    Ok(())
}

fn merged_config_overrides(
    mut root: CliConfigOverrides,
    mut command: CliConfigOverrides,
) -> CliConfigOverrides {
    command
        .raw_overrides
        .splice(0..0, root.raw_overrides.drain(..));
    command
}

fn resolve_output_path(
    output: Option<PathBuf>,
    config_toml: &ConfigToml,
) -> Result<AbsolutePathBuf> {
    if let Some(output) = output {
        return AbsolutePathBuf::relative_to_current_dir(output)
            .context("invalid --output path for builtin context lock");
    }

    if let Some(lock_config) = config_toml.builtin_context_lock.as_ref() {
        return Ok(lock_config.path.clone());
    }

    bail!(
        "no builtin_context_lock.path is configured; pass --output <path> to choose where to write the generated lock"
    );
}

fn model_catalog_base_instructions(config_toml: &ConfigToml) -> Result<String> {
    let model = configured_model(config_toml)?;
    let model_info = model_info_from_bundled_catalog(&model, config_toml)?;
    Ok(model_info.get_model_instructions(configured_personality(config_toml)?))
}

fn configured_model(config_toml: &ConfigToml) -> Result<String> {
    let profile = config_toml.get_config_profile(/*override_profile*/ None)?;
    if let Some(model) = profile.model.or_else(|| config_toml.model.clone()) {
        return Ok(model);
    }

    let mut models = bundled_models_response()
        .context("load bundled model catalog")?
        .models;
    models.sort_by_key(|left| left.priority);
    let presets: Vec<ModelPreset> = models
        .into_iter()
        .map(Into::into)
        .filter(|preset: &ModelPreset| preset.supported_in_api)
        .collect();
    Ok(presets
        .iter()
        .find(|preset| preset.show_in_picker)
        .or_else(|| presets.first())
        .map(|preset| preset.model.clone())
        .unwrap_or_default())
}

fn configured_personality(
    config_toml: &ConfigToml,
) -> Result<Option<codex_config::types::Personality>> {
    let profile = config_toml.get_config_profile(/*override_profile*/ None)?;
    let explicit_personality = profile.personality.or(config_toml.personality);
    if explicit_personality.is_some() {
        return Ok(explicit_personality);
    }

    let features = Features::from_sources(
        FeatureConfigSource {
            features: config_toml.features.as_ref(),
            include_apply_patch_tool: None,
            experimental_use_freeform_apply_patch: config_toml
                .experimental_use_freeform_apply_patch,
            experimental_use_unified_exec_tool: config_toml.experimental_use_unified_exec_tool,
        },
        FeatureConfigSource {
            features: profile.features.as_ref(),
            include_apply_patch_tool: profile.include_apply_patch_tool,
            experimental_use_freeform_apply_patch: profile.experimental_use_freeform_apply_patch,
            experimental_use_unified_exec_tool: profile.experimental_use_unified_exec_tool,
        },
        FeatureOverrides::default(),
    );
    Ok(features
        .enabled(Feature::Personality)
        .then_some(codex_config::types::Personality::Pragmatic))
}

fn model_info_from_bundled_catalog(model: &str, config_toml: &ConfigToml) -> Result<ModelInfo> {
    let candidates = bundled_models_response()
        .context("load bundled model catalog")?
        .models;
    let mut best: Option<ModelInfo> = None;
    for candidate in candidates {
        if !model.starts_with(&candidate.slug) {
            continue;
        }
        let is_better = best
            .as_ref()
            .is_none_or(|current| candidate.slug.len() > current.slug.len());
        if is_better {
            best = Some(candidate);
        }
    }
    let model_info =
        best.unwrap_or_else(|| codex_models_manager::model_info::model_info_from_slug(model));
    Ok(codex_models_manager::model_info::with_config_overrides(
        model_info,
        &ModelsManagerConfig {
            personality_enabled: configured_personality(config_toml)?.is_some(),
            ..Default::default()
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_config::config_toml::BuiltinContextLockToml;
    use pretty_assertions::assert_eq;

    #[test]
    fn explicit_output_wins_over_configured_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let configured = AbsolutePathBuf::from_absolute_path(tmp.path().join("configured.json"))
            .expect("absolute path");
        let config_toml = ConfigToml {
            builtin_context_lock: Some(BuiltinContextLockToml { path: configured }),
            ..Default::default()
        };
        let explicit = tmp.path().join("explicit.json");

        assert_eq!(
            resolve_output_path(Some(explicit.clone()), &config_toml).expect("output path"),
            AbsolutePathBuf::from_absolute_path(explicit).expect("absolute path")
        );
    }

    #[test]
    fn configured_path_is_default_output() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let configured = AbsolutePathBuf::from_absolute_path(tmp.path().join("configured.json"))
            .expect("absolute path");
        let config_toml = ConfigToml {
            builtin_context_lock: Some(BuiltinContextLockToml {
                path: configured.clone(),
            }),
            ..Default::default()
        };

        assert_eq!(
            resolve_output_path(None, &config_toml).expect("output path"),
            configured
        );
    }

    #[test]
    fn missing_output_and_configured_path_fails_clearly() {
        let err = resolve_output_path(None, &ConfigToml::default())
            .expect_err("missing output should fail");

        assert!(
            err.to_string()
                .contains("pass --output <path> to choose where to write")
        );
    }
}
