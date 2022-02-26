use std::sync::Arc;

use crate::{editor::Editor, prelude::*};

pub(crate) struct PluginCache {
    pub info: Info,
    pub params: Arc<dyn PluginParameters>,
    pub editor: Option<Box<dyn Editor>>,
}

impl PluginCache {
    pub fn new(info: &Info, params: Arc<dyn PluginParameters>, editor: Option<Box<dyn Editor>>) -> Self {
        Self {
            info: info.clone(),
            params,
            editor,
        }
    }
}
