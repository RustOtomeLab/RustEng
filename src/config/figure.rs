use crate::config::ENGINE_CONFIG;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

lazy_static::lazy_static! {
    pub static ref FIGURE_CONFIG: FigureConfig = load_figure();
}

#[derive(Debug, Deserialize, Serialize)]
struct Face {
    name: String,
    x: f32,
    y: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct FaceWrapper {
    cast: Vec<Face>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Body {
    name: String,
    rate: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct BodyWrapper {
    cast: Vec<Body>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FigureConfig {
    body_list: HashMap<String, HashMap<String, f32>>,
    face_list: HashMap<String, HashMap<String, (f32, f32)>>,
}

impl FigureConfig {
    pub fn find(
        &self,
        name: &str,
    ) -> (
        Option<&HashMap<String, f32>>,
        Option<&HashMap<String, (f32, f32)>>,
    ) {
        (self.body_list.get(name), self.face_list.get(name))
    }
}

fn load_figure() -> FigureConfig {
    let mut body_list = HashMap::new();
    let mut face_list = HashMap::new();
    for char in &ENGINE_CONFIG.character.list {
        let content =
            fs::read_to_string(format!("{}{}/face.toml", ENGINE_CONFIG.figure_path(), char))
                .unwrap();
        let item: FaceWrapper = toml::from_str(&content).unwrap();
        face_list.insert(
            char.to_string(),
            item.cast
                .into_iter()
                .map(|face| (face.name, (face.x, face.y)))
                .collect(),
        );
        let content =
            fs::read_to_string(format!("{}{}/body.toml", ENGINE_CONFIG.figure_path(), char))
                .unwrap();
        let item: BodyWrapper = toml::from_str(&content).unwrap();
        body_list.insert(
            char.to_string(),
            item.cast
                .into_iter()
                .map(|body| (body.name, body.rate))
                .collect(),
        );
    }

    FigureConfig {
        body_list,
        face_list,
    }
}
