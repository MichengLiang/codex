use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io;

const SCHEMA_VERSION: u32 = 1;

pub const BASE_INSTRUCTIONS_MODEL_CATALOG_CURRENT_ID: &str =
    "builtin.base_instructions.model_catalog.current";
pub const TOOL_EXEC_COMMAND_ID: &str = "builtin.tool.exec_command";
pub const TOOL_WRITE_STDIN_ID: &str = "builtin.tool.write_stdin";
pub const TOOL_SHELL_ID: &str = "builtin.tool.shell";
pub const TOOL_LOCAL_SHELL_ID: &str = "builtin.tool.local_shell";
pub const TOOL_APPLY_PATCH_ID: &str = "builtin.tool.apply_patch";
pub const TOOL_UPDATE_PLAN_ID: &str = "builtin.tool.update_plan";
pub const TOOL_REQUEST_USER_INPUT_ID: &str = "builtin.tool.request_user_input";
pub const TOOL_REQUEST_PERMISSIONS_ID: &str = "builtin.tool.request_permissions";
pub const TOOL_LIST_MCP_RESOURCES_ID: &str = "builtin.tool.list_mcp_resources";
pub const TOOL_LIST_MCP_RESOURCE_TEMPLATES_ID: &str = "builtin.tool.list_mcp_resource_templates";
pub const TOOL_READ_MCP_RESOURCE_ID: &str = "builtin.tool.read_mcp_resource";
pub const TOOL_GET_GOAL_ID: &str = "builtin.tool.get_goal";
pub const TOOL_CREATE_GOAL_ID: &str = "builtin.tool.create_goal";
pub const TOOL_UPDATE_GOAL_ID: &str = "builtin.tool.update_goal";
pub const TOOL_SPAWN_AGENT_ID: &str = "builtin.tool.spawn_agent";
pub const TOOL_SEND_INPUT_ID: &str = "builtin.tool.send_input";
pub const TOOL_SEND_MESSAGE_ID: &str = "builtin.tool.send_message";
pub const TOOL_FOLLOWUP_TASK_ID: &str = "builtin.tool.followup_task";
pub const TOOL_RESUME_AGENT_ID: &str = "builtin.tool.resume_agent";
pub const TOOL_WAIT_AGENT_ID: &str = "builtin.tool.wait_agent";
pub const TOOL_CLOSE_AGENT_ID: &str = "builtin.tool.close_agent";
pub const TOOL_LIST_AGENTS_ID: &str = "builtin.tool.list_agents";
pub const TOOL_SPAWN_AGENTS_ON_CSV_ID: &str = "builtin.tool.spawn_agents_on_csv";
pub const TOOL_REPORT_AGENT_JOB_RESULT_ID: &str = "builtin.tool.report_agent_job_result";
pub const TOOL_VIEW_IMAGE_ID: &str = "builtin.tool.view_image";
pub const TOOL_WEB_SEARCH_ID: &str = "builtin.tool.web_search";
pub const TOOL_IMAGE_GENERATION_ID: &str = "builtin.tool.image_generation";

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
    TOOL_EXEC_COMMAND_ID,
    TOOL_WRITE_STDIN_ID,
    TOOL_SHELL_ID,
    TOOL_LOCAL_SHELL_ID,
    TOOL_APPLY_PATCH_ID,
    TOOL_UPDATE_PLAN_ID,
    TOOL_REQUEST_USER_INPUT_ID,
    TOOL_REQUEST_PERMISSIONS_ID,
    TOOL_LIST_MCP_RESOURCES_ID,
    TOOL_LIST_MCP_RESOURCE_TEMPLATES_ID,
    TOOL_READ_MCP_RESOURCE_ID,
    TOOL_GET_GOAL_ID,
    TOOL_CREATE_GOAL_ID,
    TOOL_UPDATE_GOAL_ID,
    TOOL_SPAWN_AGENT_ID,
    TOOL_SEND_INPUT_ID,
    TOOL_SEND_MESSAGE_ID,
    TOOL_FOLLOWUP_TASK_ID,
    TOOL_RESUME_AGENT_ID,
    TOOL_WAIT_AGENT_ID,
    TOOL_CLOSE_AGENT_ID,
    TOOL_LIST_AGENTS_ID,
    TOOL_SPAWN_AGENTS_ON_CSV_ID,
    TOOL_REPORT_AGENT_JOB_RESULT_ID,
    TOOL_VIEW_IMAGE_ID,
    TOOL_WEB_SEARCH_ID,
    TOOL_IMAGE_GENERATION_ID,
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
