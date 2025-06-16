use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum KeybindAction {
    NewProject,
    OpenProject,
    SaveProject,
    CloseTab,
    CloseAllTabs,
    ImportMap,
    ExportMap,
    OpenSettings,
    Undo,
    Redo,
    Copy,
    Paste,
    Draw,
    Fill,
    Erase,
    ReloadMap,
}

#[derive(Debug, Serialize, Hash, Eq, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Keybind {
    pub key: String,
    pub with_alt: bool,
    pub with_shift: bool,
    pub with_ctrl: bool,
    pub action: Option<KeybindAction>,
}

impl Keybind {
    pub fn single(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            with_alt: false,
            with_shift: false,
            with_ctrl: false,
            action: None,
        }
    }

    pub fn with_alt(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            with_alt: true,
            with_shift: false,
            with_ctrl: false,
            action: None,
        }
    }

    pub fn with_ctrl(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            with_alt: false,
            with_shift: false,
            with_ctrl: true,
            action: None,
        }
    }

    pub fn with_shift(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            with_alt: false,
            with_shift: true,
            with_ctrl: false,
            action: None,
        }
    }

    pub fn with_ctrl_alt(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            with_alt: true,
            with_shift: false,
            with_ctrl: true,
            action: None,
        }
    }

    pub fn action(mut self, action: KeybindAction) -> Self {
        self.action = Some(action);
        self
    }
}
