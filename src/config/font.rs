use fontdb::Database;
use std::collections::BTreeSet;

lazy_static::lazy_static! {
    pub(crate) static ref SYSTEM_FONTS: SystemFonts = SystemFonts::load();
}

pub(crate) struct SystemFonts {
    families: Vec<String>,
}

impl SystemFonts {
    fn load() -> Self {
        let mut db = Database::new();
        db.load_system_fonts();

        let mut set = BTreeSet::new();
        for face in db.faces() {
            if !supports_chinese(&db, face.id) {
                continue;
            }
            if let Some((name, _)) = face.families.first() {
                set.insert(name.clone());
            }
        }

        SystemFonts {
            families: set.into_iter().collect(),
        }
    }
    
    pub(crate) fn families(&self) -> &[String] {
        &self.families
    }

    fn contains(&self, name: &str) -> bool {
        self.families.iter().any(|f| f == name)
    }
    
    pub(crate) fn default_font(&self) -> String {
        let candidates: &[&str] = if cfg!(target_os = "macos") {
            &["PingFang SC", "Hiragino Sans GB", "STHeiti", "Helvetica"]
        } else if cfg!(target_os = "windows") {
            &["Microsoft YaHei", "SimSun", "SimHei", "Segoe UI"]
        } else {
            &["Noto Sans CJK SC", "Source Han Sans SC", "WenQuanYi Micro Hei", "DejaVu Sans"]
        };

        candidates
            .iter()
            .find(|c| self.contains(c))
            .map(|c| c.to_string())
            .or_else(|| self.families.first().cloned())
            .unwrap_or_default()
    }
    
    pub(crate) fn resolve(&self, configured: &str) -> String {
        if !configured.is_empty() && self.contains(configured) {
            configured.to_string()
        } else {
            self.default_font()
        }
    }
}

fn supports_chinese(db: &Database, id: fontdb::ID) -> bool {
    // '中' 为最具代表性的常用汉字，命中即认为支持中文
    const PROBE: char = '中';
    db.with_face_data(id, |data, index| {
        ttf_parser::Face::parse(data, index)
            .map(|face| face.glyph_index(PROBE).is_some())
            .unwrap_or(false)
    })
        .unwrap_or(false)
}
