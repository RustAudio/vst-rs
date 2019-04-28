use std::sync::Arc;

use editor::Editor;
use plugin::{Info, PluginParameters};

pub(crate) struct PluginCache {
    pub info: Info,
    pub params: Arc<dyn PluginParameters>,
    pub editor: Option<Box<dyn Editor>>
}

impl PluginCache {
    pub fn new(
        info: &Info,
        params: Arc<dyn PluginParameters>,
        editor: Option<Box<dyn Editor>>
    ) -> Self {
        Self {
            info: info.clone(),
            params,
            editor
        }
    }
}
