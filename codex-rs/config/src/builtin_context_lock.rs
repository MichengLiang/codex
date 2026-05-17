use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io;

const SCHEMA_VERSION: u32 = 1;

pub const BASE_INSTRUCTIONS_MODEL_CATALOG_CURRENT_ID: &str =
    "builtin.base_instructions.model_catalog.current";

const KNOWN_BASE_INSTRUCTIONS_IDS: &[&str] = &[BASE_INSTRUCTIONS_MODEL_CATALOG_CURRENT_ID];
const KNOWN_FRAGMENT_IDS: &[&str] = &[
    "builtin.fragment.permissions_instructions",
    "builtin.fragment.collaboration_mode_instructions",
    "builtin.fragment.model_switch",
    "builtin.fragment.realtime_start",
    "builtin.fragment.personality_spec",
    "builtin.fragment.apps_instructions",
    "builtin.fragment.available_skills_scaffold",
    "builtin.fragment.available_plugins_instructions",
    "builtin.fragment.environment_context",
    "builtin.fragment.multi_agent_usage_hint",
];
const KNOWN_TOOL_IDS: &[&str] = &[
    "builtin.tool.exec_command",
    "builtin.tool.write_stdin",
    "builtin.tool.shell",
    "builtin.tool.local_shell",
    "builtin.tool.apply_patch",
    "builtin.tool.update_plan",
    "builtin.tool.request_user_input",
    "builtin.tool.list_mcp_resources",
    "builtin.tool.list_mcp_resource_templates",
    "builtin.tool.read_mcp_resource",
    "builtin.tool.get_goal",
    "builtin.tool.create_goal",
    "builtin.tool.update_goal",
    "builtin.tool.spawn_agent",
    "builtin.tool.send_input",
    "builtin.tool.resume_agent",
    "builtin.tool.wait_agent",
    "builtin.tool.close_agent",
    "builtin.tool.view_image",
    "builtin.tool.web_search",
    "builtin.tool.image_generation",
];
const KNOWN_TEMPLATE_IDS: &[&str] = &[];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuiltinContextLock {
    pub path: AbsolutePathBuf,
    pub schema_version: u32,
    pub base_instructions: BTreeMap<String, BaseInstructionsEntry>,
    pub fragments: BTreeMap<String, FragmentEntry>,
    pub tools: BTreeMap<String, ToolEntry>,
    pub templates: BTreeMap<String, TemplateEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseInstructionsEntry {
    pub id: String,
    pub enabled: bool,
    pub content: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentEntry {
    pub id: String,
    pub enabled: bool,
    pub content: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolEntry {
    pub id: String,
    pub enabled: bool,
    pub name: Option<String>,
    pub spec: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateEntry {
    pub id: String,
    pub enabled: bool,
    pub content: Option<String>,
}

pub enum BaseInstructionsDecision<'a> {
    UseContent(&'a str),
    Disable,
    Unmanaged,
}

#[derive(Debug, Deserialize)]
struct LockFile {
    schema_version: u32,
    #[serde(default)]
    base_instructions: Vec<BaseInstructionsEntry>,
    #[serde(default)]
    fragments: Vec<FragmentEntry>,
    #[serde(default)]
    tools: Vec<ToolEntry>,
    #[serde(default)]
    templates: Vec<TemplateEntry>,
}

pub async fn read_builtin_context_lock_from_path(
    fs: &dyn codex_file_system::ExecutorFileSystem,
    path: &AbsolutePathBuf,
    startup_warnings: &mut Vec<String>,
) -> io::Result<BuiltinContextLock> {
    let contents = fs
        .read_file_text(path, /*sandbox*/ None)
        .await
        .map_err(|err| {
            io::Error::new(
                err.kind(),
                format!(
                    "failed to read builtin_context_lock.path {}: {err}",
                    path.display()
                ),
            )
        })?;

    let lock_file = serde_json::from_str::<LockFile>(&contents).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "failed to parse builtin_context_lock.path {} as JSON: {err}",
                path.display()
            ),
        )
    })?;
    BuiltinContextLock::from_lock_file(path.clone(), lock_file, startup_warnings)
}

impl BuiltinContextLock {
    pub fn model_catalog_base_instructions_decision(&self) -> BaseInstructionsDecision<'_> {
        let Some(entry) = self
            .base_instructions
            .get(BASE_INSTRUCTIONS_MODEL_CATALOG_CURRENT_ID)
        else {
            return BaseInstructionsDecision::Unmanaged;
        };

        if !entry.enabled {
            return BaseInstructionsDecision::Disable;
        }

        BaseInstructionsDecision::UseContent(
            entry
                .content
                .as_deref()
                .expect("enabled base instructions lock entries are validated during parsing"),
        )
    }

    fn from_lock_file(
        path: AbsolutePathBuf,
        lock_file: LockFile,
        startup_warnings: &mut Vec<String>,
    ) -> io::Result<Self> {
        if lock_file.schema_version != SCHEMA_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unsupported builtin context lock schema_version {}; supported version is {SCHEMA_VERSION}",
                    lock_file.schema_version
                ),
            ));
        }

        let base_instructions = known_entries(
            "base_instructions",
            lock_file.base_instructions,
            KNOWN_BASE_INSTRUCTIONS_IDS,
            startup_warnings,
        )?;
        validate_base_instructions(&base_instructions)?;

        Ok(Self {
            path,
            schema_version: lock_file.schema_version,
            base_instructions,
            fragments: known_entries(
                "fragments",
                lock_file.fragments,
                KNOWN_FRAGMENT_IDS,
                startup_warnings,
            )?,
            tools: known_entries("tools", lock_file.tools, KNOWN_TOOL_IDS, startup_warnings)?,
            templates: known_entries(
                "templates",
                lock_file.templates,
                KNOWN_TEMPLATE_IDS,
                startup_warnings,
            )?,
        })
    }
}

trait LockEntry {
    fn id(&self) -> &str;
}

impl LockEntry for BaseInstructionsEntry {
    fn id(&self) -> &str {
        &self.id
    }
}

impl LockEntry for FragmentEntry {
    fn id(&self) -> &str {
        &self.id
    }
}

impl LockEntry for ToolEntry {
    fn id(&self) -> &str {
        &self.id
    }
}

impl LockEntry for TemplateEntry {
    fn id(&self) -> &str {
        &self.id
    }
}

fn known_entries<T>(
    section: &str,
    entries: Vec<T>,
    known_ids: &[&str],
    startup_warnings: &mut Vec<String>,
) -> io::Result<BTreeMap<String, T>>
where
    T: LockEntry,
{
    let mut known_entries = BTreeMap::new();
    for entry in entries {
        let id = entry.id().to_string();
        if !known_ids.contains(&id.as_str()) {
            let message = format!("Ignoring unknown builtin context lock id `{id}` in `{section}`");
            tracing::warn!("{message}");
            startup_warnings.push(message);
            continue;
        }
        if known_entries.insert(id.clone(), entry).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("duplicate builtin context lock id `{id}` in `{section}`"),
            ));
        }
    }
    Ok(known_entries)
}

fn validate_base_instructions(entries: &BTreeMap<String, BaseInstructionsEntry>) -> io::Result<()> {
    for entry in entries.values() {
        if entry.enabled && entry.content.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "builtin context lock id `{}` in `base_instructions` must provide `content` when enabled",
                    entry.id
                ),
            ));
        }
    }
    Ok(())
}
