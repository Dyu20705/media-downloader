use crate::diagnostics::DiagnosticsBuffer;
use crate::tool_manager::ToolManager;
use crate::types::{AppSettings, ToolHealth, ToolStatusInfo};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ResolvedTool {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
}

#[derive(Debug)]
pub struct ToolResolver {
    manager: ToolManager,
}

impl ToolResolver {
    pub fn new() -> Self {
        let diag = Arc::new(DiagnosticsBuffer::new());
        Self {
            manager: ToolManager::new(None, diag),
        }
    }

    pub fn with_manager(manager: ToolManager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &ToolManager {
        &self.manager
    }

    pub async fn resolve_tool(&self, tool_name: &str) -> Option<ResolvedTool> {
        let resolved = self.manager.resolve_tool(tool_name, None).await?;
        Some(ResolvedTool {
            name: resolved.name,
            path: resolved.path,
            version: resolved.version,
        })
    }

    pub async fn resolve_tool_with_settings(
        &self,
        tool_name: &str,
        settings: Option<&AppSettings>,
    ) -> Option<ResolvedTool> {
        let resolved = self.manager.resolve_tool(tool_name, settings).await?;
        Some(ResolvedTool {
            name: resolved.name,
            path: resolved.path,
            version: resolved.version,
        })
    }

    pub async fn get_all_tool_statuses(
        &self,
        settings: Option<&AppSettings>,
    ) -> Vec<ToolStatusInfo> {
        self.manager.get_all_tool_statuses(settings).await
    }

    pub async fn get_all_tools_health_with_settings(
        &self,
        settings: Option<&AppSettings>,
    ) -> Vec<ToolHealth> {
        self.manager
            .get_all_tools_health_with_settings(settings)
            .await
    }

    pub async fn get_all_tools_health(&self) -> Vec<ToolHealth> {
        self.get_all_tools_health_with_settings(None).await
    }

    pub async fn check_health(&self) -> Vec<ToolHealth> {
        self.get_all_tools_health().await
    }

    pub async fn install_tool(&self, tool_name: &str) -> Result<ToolStatusInfo, String> {
        self.manager.install_tool(tool_name).await
    }

    pub async fn repair_tool(&self, tool_name: &str) -> Result<ToolStatusInfo, String> {
        self.manager.repair_tool(tool_name).await
    }

    pub async fn install_all_missing(&self) -> Result<Vec<ToolStatusInfo>, String> {
        self.manager.install_all_missing().await
    }

    pub async fn auto_bootstrap_required_tools(&self) -> Result<Vec<ToolStatusInfo>, String> {
        self.manager.auto_bootstrap_required_tools().await
    }

    pub fn clear_cache(&self) {
        self.manager.clear_cache();
    }
}

impl Default for ToolResolver {
    fn default() -> Self {
        Self::new()
    }
}
