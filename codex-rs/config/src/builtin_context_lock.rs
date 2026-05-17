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
pub const FRAGMENT_PERMISSIONS_INSTRUCTIONS_ID: &str = "builtin.fragment.permissions_instructions";
pub const FRAGMENT_COLLABORATION_MODE_INSTRUCTIONS_ID: &str =
    "builtin.fragment.collaboration_mode_instructions";
pub const FRAGMENT_MODEL_SWITCH_ID: &str = "builtin.fragment.model_switch";
pub const FRAGMENT_REALTIME_START_ID: &str = "builtin.fragment.realtime_start";
pub const FRAGMENT_PERSONALITY_SPEC_ID: &str = "builtin.fragment.personality_spec";
pub const FRAGMENT_APPS_INSTRUCTIONS_ID: &str = "builtin.fragment.apps_instructions";
pub const FRAGMENT_AVAILABLE_SKILLS_SCAFFOLD_ID: &str =
    "builtin.fragment.available_skills_scaffold";
pub const FRAGMENT_AVAILABLE_PLUGINS_INSTRUCTIONS_ID: &str =
    "builtin.fragment.available_plugins_instructions";
pub const FRAGMENT_ENVIRONMENT_CONTEXT_ID: &str = "builtin.fragment.environment_context";
pub const FRAGMENT_MULTI_AGENT_USAGE_HINT_ID: &str = "builtin.fragment.multi_agent_usage_hint";

const KNOWN_BASE_INSTRUCTIONS_IDS: &[&str] = &[BASE_INSTRUCTIONS_MODEL_CATALOG_CURRENT_ID];
const KNOWN_FRAGMENT_IDS: &[&str] = &[
    FRAGMENT_PERMISSIONS_INSTRUCTIONS_ID,
    FRAGMENT_COLLABORATION_MODE_INSTRUCTIONS_ID,
    FRAGMENT_MODEL_SWITCH_ID,
    FRAGMENT_REALTIME_START_ID,
    FRAGMENT_PERSONALITY_SPEC_ID,
    FRAGMENT_APPS_INSTRUCTIONS_ID,
    FRAGMENT_AVAILABLE_SKILLS_SCAFFOLD_ID,
    FRAGMENT_AVAILABLE_PLUGINS_INSTRUCTIONS_ID,
    FRAGMENT_ENVIRONMENT_CONTEXT_ID,
    FRAGMENT_MULTI_AGENT_USAGE_HINT_ID,
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
const KNOWN_TOOL_NAMES: &[(&str, &str)] = &[
    (TOOL_EXEC_COMMAND_ID, "exec_command"),
    (TOOL_WRITE_STDIN_ID, "write_stdin"),
    (TOOL_SHELL_ID, "shell"),
    (TOOL_LOCAL_SHELL_ID, "local_shell"),
    (TOOL_APPLY_PATCH_ID, "apply_patch"),
    (TOOL_UPDATE_PLAN_ID, "update_plan"),
    (TOOL_REQUEST_USER_INPUT_ID, "request_user_input"),
    (TOOL_REQUEST_PERMISSIONS_ID, "request_permissions"),
    (TOOL_LIST_MCP_RESOURCES_ID, "list_mcp_resources"),
    (
        TOOL_LIST_MCP_RESOURCE_TEMPLATES_ID,
        "list_mcp_resource_templates",
    ),
    (TOOL_READ_MCP_RESOURCE_ID, "read_mcp_resource"),
    (TOOL_GET_GOAL_ID, "get_goal"),
    (TOOL_CREATE_GOAL_ID, "create_goal"),
    (TOOL_UPDATE_GOAL_ID, "update_goal"),
    (TOOL_SPAWN_AGENT_ID, "spawn_agent"),
    (TOOL_SEND_INPUT_ID, "send_input"),
    (TOOL_SEND_MESSAGE_ID, "send_message"),
    (TOOL_FOLLOWUP_TASK_ID, "followup_task"),
    (TOOL_RESUME_AGENT_ID, "resume_agent"),
    (TOOL_WAIT_AGENT_ID, "wait_agent"),
    (TOOL_CLOSE_AGENT_ID, "close_agent"),
    (TOOL_LIST_AGENTS_ID, "list_agents"),
    (TOOL_SPAWN_AGENTS_ON_CSV_ID, "spawn_agents_on_csv"),
    (TOOL_REPORT_AGENT_JOB_RESULT_ID, "report_agent_job_result"),
    (TOOL_VIEW_IMAGE_ID, "view_image"),
    (TOOL_WEB_SEARCH_ID, "web_search"),
    (TOOL_IMAGE_GENERATION_ID, "image_generation"),
];

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

pub enum FragmentDecision<'a> {
    UseOriginal,
    UseContent(&'a str),
    Disable,
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

    pub fn fragment_decision(&self, id: &str) -> FragmentDecision<'_> {
        let Some(entry) = self.fragments.get(id) else {
            return FragmentDecision::UseOriginal;
        };

        if !entry.enabled {
            return FragmentDecision::Disable;
        }

        match entry.content.as_deref() {
            Some(content) => FragmentDecision::UseContent(content),
            None => FragmentDecision::UseOriginal,
        }
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
        let tools = known_entries("tools", lock_file.tools, KNOWN_TOOL_IDS, startup_warnings)?;
        validate_tools(&tools)?;

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
            tools,
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

fn validate_tools(entries: &BTreeMap<String, ToolEntry>) -> io::Result<()> {
    for entry in entries.values() {
        let expected_name = expected_tool_name(entry.id.as_str()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "builtin context lock id `{}` in `tools` is known but has no expected tool name",
                    entry.id
                ),
            )
        })?;
        if let Some(name) = entry.name.as_deref() {
            validate_tool_name(entry.id.as_str(), expected_name, name)?;
        }
        if entry.enabled
            && let Some(spec) = entry.spec.as_ref()
        {
            let spec_name = tool_spec_name(entry.id.as_str(), spec)?;
            validate_tool_name(entry.id.as_str(), expected_name, spec_name)?;
        }
    }
    Ok(())
}

fn expected_tool_name(id: &str) -> Option<&'static str> {
    KNOWN_TOOL_NAMES
        .iter()
        .find_map(|(known_id, name)| (*known_id == id).then_some(*name))
}

fn validate_tool_name(id: &str, expected_name: &str, actual_name: &str) -> io::Result<()> {
    if actual_name == expected_name {
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "builtin context lock id `{id}` in `tools` expected tool name `{expected_name}` but lock payload names `{actual_name}`"
        ),
    ))
}

fn tool_spec_name<'a>(id: &str, spec: &'a serde_json::Value) -> io::Result<&'a str> {
    let spec_object = spec.as_object().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("builtin context lock id `{id}` in `tools` has a spec that must be an object"),
        )
    })?;
    let spec_type = spec_object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "builtin context lock id `{id}` in `tools` has a spec that must include string field `type`"
                ),
            )
        })?;

    match spec_type {
        "function" | "custom" => spec_object
            .get("name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "builtin context lock id `{id}` in `tools` has a `{spec_type}` spec that must include string field `name`"
                    ),
                )
            }),
        "local_shell" => Ok("local_shell"),
        "web_search" => Ok("web_search"),
        "image_generation" => Ok("image_generation"),
        "tool_search" => Ok("tool_search"),
        "namespace" => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "builtin context lock id `{id}` in `tools` must not use a namespace tool spec"
            ),
        )),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "builtin context lock id `{id}` in `tools` has unsupported tool spec type `{other}`"
            ),
        )),
    }
}
