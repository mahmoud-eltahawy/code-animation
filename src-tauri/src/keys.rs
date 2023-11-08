use std::collections::HashMap;

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};

pub struct Keys {
    pub keys_map: HashMap<&'static str, HotKey>,
    manager: GlobalHotKeyManager,
}

impl Keys {
    #[inline(always)]
    pub fn prepare() -> Result<Self, Box<dyn std::error::Error>> {
        let keys = Keys::init()?;
        keys.register()?;
        Ok(keys)
    }

    fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = GlobalHotKeyManager::new()?;
        let keys_map = HashMap::from([
            ("open_lesson", HotKey::new(None, Code::KeyO)),
            ("quit_lesson", HotKey::new(None, Code::KeyQ)),
            ("font_increase", HotKey::new(None, Code::Equal)),
            ("font_decrease", HotKey::new(None, Code::Minus)),
            ("next_snippet", HotKey::new(None, Code::KeyL)),
            ("previous_snippet", HotKey::new(None, Code::KeyH)),
            ("remember_toggle", HotKey::new(None, Code::KeyM)),
            (
                "next_snippet_stacked",
                HotKey::new(Some(Modifiers::SHIFT), Code::KeyL),
            ),
        ]);
        let keys = Keys { keys_map, manager };
        Ok(keys)
    }
    fn register(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.manager
            .register_all(&self.keys_map.values().cloned().collect::<Vec<_>>())?;
        Ok(())
    }
}
