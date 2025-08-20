use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::config::ENGINE_CONFIG;

lazy_static::lazy_static! {
    pub static ref FIGURE_CONFIG: FigureConfig = load_figure();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FigureConfig {
    body_list: HashMap<String, HashMap<String, f32>>,
    face_list: HashMap<String, HashMap<String, (f32, f32)>>,
}

fn load_figure() -> FigureConfig {
    let mut body_list= HashMap::new();
    let mut face_list = HashMap::new();
    for char in &ENGINE_CONFIG.character.list {
        let content = fs::read_to_string(format!("{}{}/face.toml", ENGINE_CONFIG.figure_path(), char)).unwrap();
        let item: HashMap<String, (f32, f32)> = toml::from_str(&content).unwrap();
        face_list.insert(char.to_string(), item);
        let content = fs::read_to_string(format!("{}{}/body.toml", ENGINE_CONFIG.figure_path(), char)).unwrap();
        let item: HashMap<String, f32> = toml::from_str(&content).unwrap();
        body_list.insert(char.to_string(), item);
    }

    FigureConfig { body_list, face_list }
}
