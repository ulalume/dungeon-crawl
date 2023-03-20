use crate::dungeon::DungeonLevel;
use crate::position::Position;
use ::serde_json::{from_str, from_value, json, to_string, Value};

pub fn save_game(player_position: Position, dungeon_level: DungeonLevel) {
    let json_container = json!({
        "player_position": player_position,
        "dungeon_level": dungeon_level,
    });

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    std::fs::write("save.json", to_string(&json_container).unwrap()).unwrap();

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        local_storage
            .set_item("save_data", &to_string(&json_container).expect("error"))
            .unwrap();
    }
}

pub fn load_game() -> Option<(DungeonLevel, Position)> {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    let json_str = std::fs::read_to_string("save.json").ok();

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    let json_str = {
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        local_storage.get_item("save_data").unwrap()
    };

    match json_str {
        Some(json_str) => {
            let mut json_container: Value = from_str(&json_str).unwrap();

            let dungeon_level: DungeonLevel =
                from_value(json_container.get_mut("dungeon_level").unwrap().take()).unwrap();
            let player_position: Position =
                from_value(json_container.get_mut("player_position").unwrap().take()).unwrap();

            Some((dungeon_level, player_position))
        }
        _ => None,
    }
}
