//! MCP effect disclosure and transport-admission policy.
//!
//! The checked-in catalogues are the source of truth. Effect annotations are
//! hints for clients; admission is enforced separately and fails closed for
//! unknown tools and for consequential tools presented over remote HTTP.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

use rmcp::model::ToolAnnotations;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Effect {
    ReadOnly,
    WriteLocalDraft,
    WriteLocalReview,
    NetworkAndLocalWrite,
    LocalWritePreview,
    LocalWrite,
}

impl Effect {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "read_only" => Ok(Self::ReadOnly),
            "write_local_draft" => Ok(Self::WriteLocalDraft),
            "write_local_review" => Ok(Self::WriteLocalReview),
            "network_and_local_write" => Ok(Self::NetworkAndLocalWrite),
            "local_write_preview" => Ok(Self::LocalWritePreview),
            "local_write" => Ok(Self::LocalWrite),
            other => Err(format!("unknown MCP effect `{other}`")),
        }
    }

    const fn is_read_only(self) -> bool {
        matches!(self, Self::ReadOnly)
    }

    const fn is_open_world(self) -> bool {
        matches!(self, Self::NetworkAndLocalWrite)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ToolEffectPolicy {
    effect: Effect,
}

impl ToolEffectPolicy {
    pub(crate) fn annotations(self) -> ToolAnnotations {
        ToolAnnotations::new()
            .read_only(self.effect.is_read_only())
            .destructive(false)
            // The effect contracts do not yet promise idempotent writes. Keep
            // the hint conservative until exact retry semantics are contracted.
            .idempotent(self.effect.is_read_only())
            .open_world(self.effect.is_open_world())
    }

    pub(crate) const fn remote_allowed(self) -> bool {
        self.effect.is_read_only()
    }
}

#[derive(Deserialize)]
struct McpToolCatalogue {
    tools: Vec<McpToolEntry>,
}

#[derive(Deserialize)]
struct McpToolEntry {
    name: String,
    effect: String,
}

#[derive(Deserialize)]
struct InterfaceCatalogue {
    entries: Vec<InterfaceEntry>,
}

#[derive(Deserialize)]
struct InterfaceEntry {
    mcp_tool: String,
}

fn registry() -> &'static BTreeMap<String, ToolEffectPolicy> {
    static REGISTRY: OnceLock<BTreeMap<String, ToolEffectPolicy>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        build_registry().unwrap_or_else(|error| panic!("invalid MCP effect catalogue: {error}"))
    })
}

fn build_registry() -> Result<BTreeMap<String, ToolEffectPolicy>, String> {
    let interface: InterfaceCatalogue =
        serde_json::from_str(include_str!("../../../contracts/interface-catalog.json"))
            .map_err(|error| format!("interface catalogue must parse: {error}"))?;
    let effects: McpToolCatalogue =
        serde_json::from_str(include_str!("../../../contracts/mcp/tool-catalog.json"))
            .map_err(|error| format!("MCP tool catalogue must parse: {error}"))?;

    build_registry_from(interface, effects)
}

fn build_registry_from(
    interface: InterfaceCatalogue,
    effects: McpToolCatalogue,
) -> Result<BTreeMap<String, ToolEffectPolicy>, String> {
    let mut interface_tools = BTreeSet::new();
    for entry in interface.entries {
        if !interface_tools.insert(entry.mcp_tool) {
            return Err("interface catalogue contains a duplicate MCP tool".to_owned());
        }
    }
    let mut registry = BTreeMap::new();
    for entry in effects.tools {
        if !interface_tools.contains(&entry.name) {
            return Err(format!(
                "MCP effect catalogue contains extra tool `{}`",
                entry.name
            ));
        }
        let effect = Effect::parse(&entry.effect)?;
        if registry
            .insert(entry.name.clone(), ToolEffectPolicy { effect })
            .is_some()
        {
            return Err(format!(
                "MCP effect catalogue contains duplicate tool `{}`",
                entry.name
            ));
        }
    }
    if registry.len() != interface_tools.len() {
        let missing = interface_tools
            .iter()
            .filter(|name| !registry.contains_key(*name))
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "MCP effect catalogue is missing interface tools: {missing}"
        ));
    }
    Ok(registry)
}

pub(crate) fn policy_for(tool_name: &str) -> Option<ToolEffectPolicy> {
    registry().get(tool_name).copied()
}

pub(crate) fn registered_policy_for(tool_name: &str) -> ToolEffectPolicy {
    policy_for(tool_name)
        .unwrap_or_else(|| panic!("registered MCP tool `{tool_name}` has no effect policy"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consequential_catalogue_tools_are_not_remote_admissible() {
        for name in [
            "plan_review",
            "press_review_strategy",
            "execute_search",
            "deduplicate_records",
            "record_screening_decision",
        ] {
            assert!(!registered_policy_for(name).remote_allowed(), "{name}");
            assert_eq!(
                registered_policy_for(name).annotations().read_only_hint,
                Some(false),
                "{name}"
            );
        }
        let execution = registered_policy_for("execute_search").annotations();
        assert_eq!(execution.open_world_hint, Some(true));
        assert_eq!(execution.idempotent_hint, Some(false));
    }

    #[test]
    fn read_only_catalogue_tools_remain_remote_admissible() {
        for name in ["validate_plan", "compile_strategy", "workflow"] {
            let policy = registered_policy_for(name);
            assert!(policy.remote_allowed(), "{name}");
            let annotations = policy.annotations();
            assert_eq!(annotations.read_only_hint, Some(true));
            assert_eq!(annotations.destructive_hint, Some(false));
            assert_eq!(annotations.idempotent_hint, Some(true));
            assert_eq!(annotations.open_world_hint, Some(false));
        }
    }

    #[test]
    fn unknown_tools_fail_closed() {
        assert!(policy_for("uncatalogued_tool").is_none());
    }

    #[test]
    fn missing_effect_entries_fail_registry_construction() {
        let result = build_registry_from(
            InterfaceCatalogue {
                entries: vec![InterfaceEntry {
                    mcp_tool: "missing".to_owned(),
                }],
            },
            McpToolCatalogue { tools: Vec::new() },
        );
        assert!(matches!(
            result,
            Err(message) if message == "MCP effect catalogue is missing interface tools: missing"
        ));
    }
}
