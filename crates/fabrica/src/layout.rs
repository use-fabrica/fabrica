use std::fs;

use ui::dock::DockAreaState;

#[cfg(debug_assertions)]
const STATE_FILE: &str = "target/docks.json";
#[cfg(not(debug_assertions))]
const STATE_FILE: &str = "docks.json";

#[allow(dead_code)]
pub(crate) trait Layout {
    fn load_layout(&mut self) -> anyhow::Result<DockAreaState> {
        let json = fs::read_to_string(STATE_FILE)?;
        let state = serde_json::from_str::<DockAreaState>(&json)?;
        Ok(state)
    }

    fn save_state(state: &DockAreaState) -> anyhow::Result<()> {
        println!("Save layout...");
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(STATE_FILE, json)?;
        Ok(())
    }
}
