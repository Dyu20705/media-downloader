use crate::types::{MediaWorkspace, WorkspaceItem};

/// P2 Extension Point: MediaWorkspace Domain Boundary
pub struct WorkspaceManager;

impl WorkspaceManager {
    pub fn create_workspace(name: &str) -> MediaWorkspace {
        let now = {
            use std::time::{SystemTime, UNIX_EPOCH};
            let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            format!("{}.{:03}Z", dur.as_secs(), dur.subsec_millis())
        };

        MediaWorkspace {
            id: format!("ws_{}", uuid_simple()),
            name: name.to_string(),
            items: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn add_item(workspace: &mut MediaWorkspace, item: WorkspaceItem) {
        workspace.items.push(item);
    }
}

fn uuid_simple() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::Instant::now().hash(&mut h);
    format!("{:08x}", h.finish())
}
