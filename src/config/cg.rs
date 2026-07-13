use crate::config::ENGINE_CONFIG;
use crate::error::{EngineError, ExecutorError};
use crate::ui::initialize::ExItem;
use serde::{Deserialize, Serialize};
use slint::{Image, ModelRc, VecModel};
use std::{cell::RefCell, path::Path, rc::Rc};
use std::{collections::HashMap, fs};

lazy_static::lazy_static! {
    pub(crate) static ref CG_CONFIG: CgConfig = load_cg();
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CgMap {
    cg: Vec<u64>,
}

impl CgMap {
    pub(crate) fn new(cg: Vec<u64>) -> Self {
        Self { cg }
    }

    pub(crate) fn cg(self) -> Vec<u64> {
        self.cg
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Length {
    name: String,
    index: usize,
    length: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct LengthWrapper {
    cast: Vec<Length>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CgConfig {
    cg_by_name: HashMap<String, (usize, u64)>,
    cg_by_id: HashMap<usize, (String, u64)>,
    length: usize,
}

impl CgConfig {
    pub(crate) fn find_by_name(&self, name: &str) -> Option<&(usize, u64)> {
        self.cg_by_name.get(name)
    }

    pub(crate) fn find_by_id(&self, index: u64) -> Option<&(String, u64)> {
        self.cg_by_id.get(&(index as usize))
    }

    pub(crate) fn length(&self) -> usize {
        self.length
    }
}

fn load_cg() -> CgConfig {
    let content = fs::read_to_string(format!("{}length.toml", ENGINE_CONFIG.cg_path(),)).unwrap();
    let name_item: LengthWrapper = toml::from_str(&content).unwrap();
    let index_item = name_item.cast.clone();
    let length = index_item.len();
    CgConfig {
        cg_by_name: name_item
            .cast
            .into_iter()
            .map(|length| (length.name, (length.index, length.length)))
            .collect(),
        cg_by_id: index_item
            .into_iter()
            .map(|length| (length.index, (length.name, length.length)))
            .collect(),
        length,
    }
}

pub(crate) fn get_cg(cg: Rc<RefCell<Vec<u64>>>) -> Result<ModelRc<ModelRc<ExItem>>, EngineError> {
    let cg_map = cg.borrow();

    let mut ex_items: Vec<ModelRc<ExItem>> = Vec::with_capacity(10);
    let mut i = 1;
    let mut ex_page: Vec<ExItem> = Vec::new();
    for cgs in cg_map.iter() {
        while i <= CG_CONFIG.length() as u64 {
            if let Some((_, length)) = CG_CONFIG.find_by_id(i) {
                let (mut images, mut l, mut is_lock) = (Vec::new(), *length, true);
                for j in 1..=*length {
                    if cgs & (1 << (j + i % 64 - 1)) != 0 {
                        if let Some((name, _)) = CG_CONFIG.find_by_id(j + i - 1) {
                            images.push(
                                Image::load_from_path(Path::new(&format!(
                                    "{}{}.png",
                                    ENGINE_CONFIG.cg_path(),
                                    name
                                )))
                                .unwrap(),
                            );
                            is_lock = false;
                        } else {
                            return Err(ExecutorError::CgMetadataMissing(j + i - 1).into());
                        }
                    } else {
                        l -= 1;
                    }
                }
                i += *length;
                let item = ExItem {
                    bg: Rc::new(VecModel::from(images)).into(),
                    indexs: l as i32,
                    is_lock,
                };
                if ex_page.len() < 16 {
                    ex_page.push(item);
                } else {
                    ex_items.push(Rc::new(VecModel::from(std::mem::take(&mut ex_page))).into());
                    ex_page.push(item);
                }
            } else {
                return Err(ExecutorError::CgMetadataMissing(i).into());
            }
        }
    }
    ex_items.push(Rc::new(VecModel::from(ex_page)).into());

    Ok(Rc::new(VecModel::from(ex_items)).into())
}
