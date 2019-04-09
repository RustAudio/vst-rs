use std::sync::Arc;

use editor::Editor;
use plugin::{Info, PluginParameters};

pub(crate) struct PluginCache {
    pub info: Info,
    pub params: Arc<PluginParameters>,
    pub editor: Option<Box<Editor>>
}

impl PluginCache {
    pub fn new(
        info: &Info,
        params: Arc<PluginParameters>,
        editor: Option<Box<Editor>>
    ) -> Self {
        Self {
            info: info.clone(),
            params,
            editor
        }
    }
}
