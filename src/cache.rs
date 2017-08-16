use plugin::Info;

pub(crate) struct PluginCache {
    pub info: Info,
}

impl PluginCache {
    pub fn new(info: &Info) -> Self {
        Self {
            info: info.clone(),
        }
    }
}