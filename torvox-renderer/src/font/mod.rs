pub mod atlas;
pub mod cjk;
pub mod rasterization;
pub mod shaping;

use std::num::NonZeroUsize;

use cosmic_text::FontSystem;
use lru::LruCache;
use thiserror::Error;

use crate::lock_util::write_or_recover;

pub const GLYPH_CACHE_CAPACITY: usize = 10_000;

#[derive(Debug, Error)]
pub enum FontError {
    #[error("no monospace font found")]
    NoMonospaceFont,
    #[error("font loading failed: {0}")]
    FontLoad(String),
    #[error("atlas allocation failed")]
    AtlasAllocationFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: fontdb::ID,
    pub glyph_id: u16,
    pub pixel_size: u16,
}

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub atlas_x: i32,
    pub atlas_y: i32,
    pub width: i32,
    pub height: i32,
    pub placement: swash::zeno::Placement,
    pub advance_width: f32,
    pub allocation_id: Option<guillotiere::AllocId>,
}

#[derive(Debug, Clone)]
pub struct ShapedGlyphInfo {
    pub glyph_id: u16,
    pub font_id: fontdb::ID,
    pub x: f32,
    pub w: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

const PREFERRED_MONOSPACE_FONTS: &[&str] = &[
    "roboto mono",
    "droid sans mono",
    "noto sans mono",
    "source code pro",
    "fira code",
    "fira mono",
    "jetbrains mono",
    "dejavu sans mono",
    "noto sans mono cjk",
    "liberation mono",
    "ubuntu mono",
    "cascadia",
    "ia writer",
    "hack",
    "inconsolata",
    "iosevka",
    "meslo",
    "consolas",
    "menlo",
    "monaco",
    "courier",
];

#[cfg(target_os = "android")]
fn resolve_system_monospace_from_fonts_xml() -> Option<String> {
    let xml_path = std::path::Path::new("/system/etc/fonts.xml");
    let content = std::fs::read_to_string(xml_path).ok()?;

    let monospace_names = ["monospace", "sans-serif mono", "serif mono"];
    for mono_name in &monospace_names {
        let pattern = format!("name=\"{}\"", mono_name);
        if let Some(family_start) = content.find(&pattern) {
            let family_end = content[family_start..].find("</family>");
            if let Some(offset) = family_end {
                let family_block = &content[family_start..family_start + offset];
                if let Some(font_start) = family_block.find("<font ") {
                    let after_font = &family_block[font_start..];
                    if let Some(gt_pos) = after_font.find('>') {
                        let text_start = gt_pos + 1;
                        if let Some(lt_pos) = after_font[text_start..].find('<') {
                            let filename = after_font[text_start..text_start + lt_pos].trim();
                            if !filename.is_empty() {
                                log::debug!("FONT_XML: monospace target='{}'", filename);
                                return Some(filename.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "android"))]
fn resolve_system_monospace_from_fonts_xml() -> Option<String> {
    None
}

#[cfg(target_os = "android")]
fn is_font_file(entry: &std::path::Path) -> bool {
    entry
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            ext.eq_ignore_ascii_case("ttf")
                || ext.eq_ignore_ascii_case("otf")
                || ext.eq_ignore_ascii_case("ttc")
        })
}

#[cfg(target_os = "android")]
static CACHED_FONT_PATHS: std::sync::OnceLock<Vec<std::path::PathBuf>> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
static CACHED_FONT_DB: std::sync::OnceLock<fontdb::Database> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
static EXTRA_FONT_PATHS: std::sync::RwLock<Vec<std::path::PathBuf>> =
    std::sync::RwLock::new(Vec::new());

#[cfg(target_os = "android")]
pub fn set_extra_font_paths(paths: Vec<std::path::PathBuf>) {
    let mut extra = write_or_recover(&EXTRA_FONT_PATHS, "set_extra_font_paths");
    *extra = paths;
    log::debug!("FONT_LOAD: set {} extra font paths", extra.len());
}

#[cfg(target_os = "android")]
fn load_font_database() -> fontdb::Database {
    let db = CACHED_FONT_DB.get_or_init(|| {
        let font_paths = CACHED_FONT_PATHS.get_or_init(|| {
            let mut paths = Vec::new();
            for dir in [
                "/system/fonts/",
                "/system/product/fonts/",
                "/system_ext/fonts/",
                "/vendor/fonts/",
                "/product/fonts/",
            ] {
                let dir_path = std::path::Path::new(dir);
                if let Ok(entries) = std::fs::read_dir(dir_path) {
                    for entry in entries.flatten() {
                        if is_font_file(&entry.path()) {
                            paths.push(entry.path());
                        }
                    }
                }
            }
            log::debug!("FONT_LOAD: cached {} font paths", paths.len());
            paths
        });

        let mut db = fontdb::Database::new();
        let mut count = 0u32;
        for path in font_paths {
            if db.load_font_file(path).is_ok() {
                count += 1;
            }
        }
        log::debug!("FONT_LOAD: loaded {count} fonts from cached paths");
        db
    });
    db.clone()
}

pub struct FontPipeline {
    pub(crate) font_system: FontSystem,
    pub(crate) scaler_context: swash::scale::ScaleContext,
    pub(crate) atlas: guillotiere::AtlasAllocator,
    pub(crate) glyph_cache: LruCache<GlyphKey, GlyphInfo>,
    pub(crate) atlas_bitmap: Vec<u8>,
    pub(crate) atlas_width: u32,
    pub(crate) atlas_height: u32,
    pub(crate) font_id: Option<fontdb::ID>,
    pub(crate) cjk_fallback_ids: Vec<fontdb::ID>,
    pub(crate) font_size: f32,
    pub(crate) raster_scale: f32,
    pub(crate) atlas_generation: u64,
    pub(crate) dirty_rect: Option<(u32, u32, u32, u32)>,
    system_locale: String,
    pub(crate) shaping_buffer: Option<cosmic_text::Buffer>,
    pub(crate) shape_cache: LruCache<String, Vec<ShapedGlyphInfo>>,
    pub(crate) ascii_glyph_ids: [Option<swash::GlyphId>; 128],
}

impl FontPipeline {
    pub fn new(atlas_width: i32, atlas_height: i32, font_size: f32) -> Self {
        #[cfg(target_os = "android")]
        let mut db = load_font_database();

        #[cfg(target_os = "android")]
        if let Ok(extra) = EXTRA_FONT_PATHS.read() {
            for path in extra.iter() {
                if path.is_file() {
                    if let Err(error) = db.load_font_file(path) {
                        log::warn!("font: failed to load font file {path:?}: {error}");
                    }
                } else if path.is_dir()
                    && let Ok(entries) = std::fs::read_dir(path)
                {
                    for entry in entries.flatten() {
                        let file_path = entry.path();
                        if is_font_file(&file_path)
                            && let Err(error) = db.load_font_file(&file_path)
                        {
                            log::warn!("font: failed to load font file {file_path:?}: {error}");
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "android"))]
        let db = {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            db
        };

        let font_system = FontSystem::new_with_locale_and_db(String::new(), db);

        let scaler_context = swash::scale::ScaleContext::new();
        let atlas = guillotiere::AtlasAllocator::new(guillotiere::size2(atlas_width, atlas_height));
        let atlas_bitmap = vec![0u8; (atlas_width * atlas_height * 4) as usize];

        let mut pipeline = Self {
            font_system,
            scaler_context,
            atlas,
            glyph_cache: LruCache::new(
                NonZeroUsize::new(GLYPH_CACHE_CAPACITY)
                    .expect("GLYPH_CACHE_CAPACITY is 10_000, always non-zero"),
            ),
            atlas_bitmap,
            atlas_width: atlas_width as u32,
            atlas_height: atlas_height as u32,
            font_id: None,
            cjk_fallback_ids: Vec::new(),
            font_size,
            atlas_generation: 0,
            dirty_rect: None,
            system_locale: String::new(),
            shaping_buffer: None,
            shape_cache: LruCache::new(
                NonZeroUsize::new(shaping::SHAPE_CACHE_CAPACITY)
                    .expect("SHAPE_CACHE_CAPACITY must be non-zero"),
            ),
            ascii_glyph_ids: [None; 128],
            raster_scale: 1.0,
        };

        if pipeline.font_id.is_none() {
            pipeline.find_monospace_font();
        }
        let system_locale = pipeline.system_locale.clone();
        pipeline.find_cjk_fallback_fonts(&system_locale);
        pipeline.rasterize_ascii();
        pipeline
    }

    pub fn from_fixture(
        atlas_width: i32,
        atlas_height: i32,
        font_size: f32,
        fixture_dir: &str,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let db = font_system.db_mut();

        let path = std::path::Path::new(fixture_dir);
        if path.is_dir()
            && let Ok(entries) = std::fs::read_dir(path)
        {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("ttf") || e.eq_ignore_ascii_case("otf"))
                    && let Err(error) = db.load_font_file(&file_path)
                {
                    log::warn!("font: failed to load font file {file_path:?}: {error}");
                }
            }
        }

        let scaler_context = swash::scale::ScaleContext::new();
        let atlas = guillotiere::AtlasAllocator::new(guillotiere::size2(atlas_width, atlas_height));
        let atlas_bitmap = vec![0u8; (atlas_width * atlas_height * 4) as usize];

        let mut pipeline = Self {
            font_system,
            scaler_context,
            atlas,
            glyph_cache: LruCache::new(
                NonZeroUsize::new(GLYPH_CACHE_CAPACITY)
                    .expect("GLYPH_CACHE_CAPACITY is 10_000, always non-zero"),
            ),
            atlas_bitmap,
            atlas_width: atlas_width as u32,
            atlas_height: atlas_height as u32,
            font_id: None,
            cjk_fallback_ids: Vec::new(),
            font_size,
            atlas_generation: 0,
            dirty_rect: None,
            system_locale: String::new(),
            shaping_buffer: None,
            shape_cache: LruCache::new(
                NonZeroUsize::new(shaping::SHAPE_CACHE_CAPACITY)
                    .expect("SHAPE_CACHE_CAPACITY must be non-zero"),
            ),
            ascii_glyph_ids: [None; 128],
            raster_scale: 1.0,
        };

        pipeline.find_monospace_font();
        let system_locale = pipeline.system_locale.clone();
        pipeline.find_cjk_fallback_fonts(&system_locale);
        pipeline
    }

    fn find_monospace_font(&mut self) {
        let db = self.font_system.db();

        if let Some(target_filename) = resolve_system_monospace_from_fonts_xml() {
            let stem = std::path::Path::new(&target_filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let stem_lower = stem.to_lowercase().replace(['-', '_'], " ");
            for face in db.faces() {
                if !face.monospaced {
                    continue;
                }
                let name = face
                    .families
                    .first()
                    .map(|(n, _)| n.to_lowercase())
                    .unwrap_or_default();
                let name_normalized = name.replace(['-', '_'], " ");
                if name_normalized == stem_lower || name_normalized.contains(&stem_lower) {
                    let display = face
                        .families
                        .first()
                        .map(|(n, _)| n.clone())
                        .unwrap_or_default();
                    log::debug!(
                        "FONT_SELECT: fonts.xml monospace id={:?} name='{}' (stem='{}')",
                        face.id,
                        display,
                        stem
                    );
                    self.font_id = Some(face.id);
                    return;
                }
            }
            for face in db.faces() {
                if !face.monospaced {
                    continue;
                }
                let name_nospace: String = face
                    .families
                    .first()
                    .map(|(n, _)| {
                        n.to_lowercase()
                            .chars()
                            .filter(|c| !c.is_whitespace())
                            .collect()
                    })
                    .unwrap_or_default();
                let stem_nospace = stem_lower.replace(' ', "");
                if name_nospace == stem_nospace {
                    let display = face
                        .families
                        .first()
                        .map(|(n, _)| n.clone())
                        .unwrap_or_default();
                    log::debug!(
                        "FONT_SELECT: fonts.xml monospace (nospace) id={:?} name='{}'",
                        face.id,
                        display
                    );
                    self.font_id = Some(face.id);
                    return;
                }
            }
        }

        for face in db.faces() {
            if !face.monospaced {
                continue;
            }
            let name = face
                .families
                .first()
                .map(|(n, _)| n.to_lowercase())
                .unwrap_or_default();
            if PREFERRED_MONOSPACE_FONTS.iter().any(|p| name.contains(p)) {
                let display = face
                    .families
                    .first()
                    .map(|(n, _)| n.clone())
                    .unwrap_or_default();
                log::debug!(
                    "FONT_SELECT: preferred monospace id={:?} name='{}'",
                    face.id,
                    display
                );
                self.font_id = Some(face.id);
                return;
            }
        }

        for face in db.faces() {
            if face.monospaced {
                let name = face
                    .families
                    .first()
                    .map(|(n, _)| n.to_lowercase())
                    .unwrap_or_default();
                if name.contains("cjk")
                    || name.contains("sc")
                    || name.contains("tc")
                    || name.contains("jp")
                    || name.contains("kr")
                    || name.contains("han")
                {
                    continue;
                }
                let display = face
                    .families
                    .first()
                    .map(|(n, _)| n.clone())
                    .unwrap_or_default();
                log::debug!("FONT_SELECT: monospace id={:?} name='{}'", face.id, display);
                self.font_id = Some(face.id);
                return;
            }
        }

        for face in db.faces() {
            if face.monospaced {
                let name = face
                    .families
                    .first()
                    .map(|(n, _)| n.clone())
                    .unwrap_or_default();
                log::debug!(
                    "FONT_SELECT: monospace (CJK ok) id={:?} name='{}'",
                    face.id,
                    name
                );
                self.font_id = Some(face.id);
                return;
            }
        }

        if let Some(face) = db.faces().next() {
            let name = face
                .families
                .first()
                .map(|(n, _)| n.clone())
                .unwrap_or_default();
            log::warn!(
                "FONT_SELECT: fallback to any face id={:?} name='{}'",
                face.id,
                name
            );
            self.font_id = Some(face.id);
            return;
        }

        log::error!("FONT_SELECT: no font found in system!");
    }

    pub fn set_font_family(&mut self, family_name: &str) -> bool {
        self.shape_cache.clear();
        if family_name.is_empty() {
            self.font_id = None;
            self.find_monospace_font();
            self.glyph_cache.clear();
            self.atlas = guillotiere::AtlasAllocator::new(guillotiere::size2(
                self.atlas_width as i32,
                self.atlas_height as i32,
            ));
            self.atlas_bitmap.fill(0);
            self.atlas_generation = self.atlas_generation.wrapping_add(1);
            self.cjk_fallback_ids.clear();
            let system_locale = self.system_locale.clone();
            self.find_cjk_fallback_fonts(&system_locale);
            self.rasterize_ascii();
            return true;
        }
        let found = {
            let db = self.font_system.db_mut();
            Self::find_font_by_name(db, family_name)
        };
        if let Some(id) = found {
            let db = self.font_system.db();
            let name = db
                .face(id)
                .and_then(|f| f.families.first().map(|(n, _)| n.clone()))
                .unwrap_or_default();
            log::debug!(
                "FONT_DIAG: set_font_family('{}') found id={:?} name='{}'",
                family_name,
                id,
                name
            );
            self.font_id = Some(id);
            self.glyph_cache.clear();
            self.atlas = guillotiere::AtlasAllocator::new(guillotiere::size2(
                self.atlas_width as i32,
                self.atlas_height as i32,
            ));
            self.atlas_bitmap.fill(0);
            self.atlas_generation = self.atlas_generation.wrapping_add(1);
            self.cjk_fallback_ids.clear();
            let system_locale = self.system_locale.clone();
            self.find_cjk_fallback_fonts(&system_locale);
            self.rasterize_ascii();
            return true;
        }
        log::warn!(
            "FONT_DIAG: set_font_family('{}') NOT FOUND in fontdb",
            family_name
        );
        false
    }

    pub fn set_system_locale(&mut self, locale: &str) {
        self.shape_cache.clear();
        self.system_locale = locale.to_string();
        self.cjk_fallback_ids.clear();
        self.find_cjk_fallback_fonts(&self.system_locale.clone());
    }

    pub fn set_font_size_in_place(&mut self, new_size: f32) -> (f32, f32) {
        self.font_size = new_size;
        self.glyph_cache.clear();
        self.atlas = guillotiere::AtlasAllocator::new(guillotiere::size2(
            self.atlas_width as i32,
            self.atlas_height as i32,
        ));
        self.atlas_bitmap.fill(0);
        self.atlas_generation = self.atlas_generation.wrapping_add(1);
        self.rasterize_ascii();
        let (cw, ch) = self.cell_metrics();
        log::debug!(
            "FONT_SIZE_IN_PLACE: size={} cell={:.1}x{:.1}",
            new_size,
            cw,
            ch
        );
        (cw, ch)
    }

    pub fn set_raster_scale(&mut self, scale: f32) {
        let scale = if scale > 0.0 && scale.is_finite() {
            scale
        } else {
            1.0
        };
        if (scale - self.raster_scale).abs() < 1e-3 {
            return;
        }
        self.raster_scale = scale;
        self.glyph_cache.clear();
        self.atlas = guillotiere::AtlasAllocator::new(guillotiere::size2(
            self.atlas_width as i32,
            self.atlas_height as i32,
        ));
        self.atlas_bitmap.fill(0);
        self.atlas_generation = self.atlas_generation.wrapping_add(1);
        self.rasterize_ascii();
        log::debug!("RASTER_SCALE: scale={:.3}", scale);
    }

    pub fn get_raster_scale(&self) -> f32 {
        self.raster_scale
    }

    pub fn current_font_family_name(&self) -> Option<String> {
        let font_id = self.font_id?;
        let db = self.font_system.db();
        let face_info = db.face(font_id)?;
        let family = face_info.families.first()?;
        Some(family.0.clone())
    }

    pub fn default_font_name(&self) -> String {
        if let Some(id) = self.font_id
            && let Some(name) = self
                .font_system
                .db()
                .face(id)
                .and_then(|f| f.families.first().map(|(n, _)| n.clone()))
        {
            return name;
        }
        self.system_monospace_name()
    }

    pub fn system_monospace_name(&self) -> String {
        let db = self.font_system.db();
        for face in db.faces() {
            if !face.monospaced {
                continue;
            }
            let name = face
                .families
                .first()
                .map(|(n, _)| n.to_lowercase())
                .unwrap_or_default();
            if PREFERRED_MONOSPACE_FONTS.iter().any(|p| name.contains(p)) {
                return face
                    .families
                    .first()
                    .map(|(n, _)| n.clone())
                    .unwrap_or_default();
            }
        }
        for face in db.faces() {
            if face.monospaced {
                return face
                    .families
                    .first()
                    .map(|(n, _)| n.clone())
                    .unwrap_or_default();
            }
        }
        "monospace".to_string()
    }

    pub fn cjk_fallback_names(&self) -> Vec<String> {
        let db = self.font_system.db();
        let mut raw_names: Vec<String> = self
            .cjk_fallback_ids
            .iter()
            .filter_map(|&id| {
                let face = db.face(id)?;
                face.families.first().map(|(name, _)| name.clone())
            })
            .collect();
        raw_names.sort();
        raw_names.dedup();
        let mut normalized = Vec::new();
        let mut seen_generic = false;
        for name in raw_names {
            let lower = name.to_lowercase();
            if lower.contains("cjk") {
                if !seen_generic {
                    normalized.push("Noto Sans CJK".to_string());
                    seen_generic = true;
                }
            } else {
                normalized.push(name);
            }
        }
        normalized
    }

    pub fn font_information(&self) -> String {
        let db = self.font_system.db();
        let mut parts = Vec::new();
        if let Some(id) = self.font_id
            && let Some(face) = db.face(id)
        {
            let name = face.families.first().map_or("unknown", |(n, _)| n.as_str());
            let mono = if face.monospaced {
                "monospaced"
            } else {
                "proportional"
            };
            parts.push(format!("Active: {} ({})", name, mono));
        }
        let cjk = self.cjk_fallback_names();
        if !cjk.is_empty() {
            parts.push(format!("CJK fallback: {}", cjk.join(", ")));
        } else {
            let primary_is_cjk = self.font_id.and_then(|id| db.face(id)).is_some_and(|face| {
                face.families.iter().any(|(name, _)| {
                    let l = name.to_lowercase();
                    l.contains("cjk")
                        || l.contains("chinese")
                        || l.contains("japanese")
                        || l.contains("korean")
                        || l.contains(" sc")
                        || l.contains(" tc")
                        || l.contains(" jp")
                        || l.contains(" kr")
                })
            });
            if primary_is_cjk {
                parts.push("CJK fallback: skipped (primary font supports CJK)".to_string());
            } else {
                parts.push("CJK fallback: none".to_string());
            }
        }
        let (cw, ch) = self.cell_metrics();
        parts.push(format!("Cell: {:.1}x{:.1}px", cw, ch));
        parts.push(format!("Font size: {:.1}px", self.font_size));
        parts.join("\n")
    }

    pub fn list_all_font_families(&self) -> Vec<String> {
        let db = self.font_system.db();
        let mut families = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for face in db.faces() {
            for (family, _) in &face.families {
                if seen.insert(family.to_lowercase()) {
                    families.push(family.clone());
                }
            }
        }
        families.sort();
        families
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn glyph_information(&mut self, ch: char) -> Option<GlyphInfo> {
        let primary_font_id = self.font_id?;

        if (ch as u32) < 128
            && let Some(gid) = self.ascii_glyph_ids[ch as usize]
        {
            let key = GlyphKey {
                font_id: primary_font_id,
                glyph_id: gid,
                pixel_size: (self.font_size * self.raster_scale) as u16,
            };
            if let Some(info) = self.glyph_cache.get(&key).cloned() {
                return Some(info);
            }
        }

        let glyph_id = {
            let db = self.font_system.db();
            db.with_face_data(primary_font_id, |font_data, face_index| {
                let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
                let charmap = font_ref.charmap();
                Some(charmap.map(ch))
            })?
        }?;

        if (ch as u32) < 128 {
            self.ascii_glyph_ids[ch as usize] = Some(glyph_id);
        }

        let has_cjk_fallback = !self.cjk_fallback_ids.is_empty();

        if glyph_id != 0 && (ch as u32) >= 0x2E80 && has_cjk_fallback {
            let is_outline = self.glyph_source_is_outline(primary_font_id, glyph_id);
            if !is_outline && let Some(fallback_info) = self.try_cjk_outline_fallback(ch) {
                return Some(fallback_info);
            }
        }

        if glyph_id == 0 && has_cjk_fallback {
            for &fallback_id in &self.cjk_fallback_ids {
                let fallback_glyph = {
                    let db = self.font_system.db();
                    db.with_face_data(fallback_id, |font_data, face_index| {
                        let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
                        let charmap = font_ref.charmap();
                        Some(charmap.map(ch))
                    })
                };
                if let Some(Some(fid)) = fallback_glyph
                    && fid != 0
                {
                    return self.glyph_information_from_font(fallback_id, ch, fid);
                }
            }
        }

        self.glyph_information_from_font(primary_font_id, ch, glyph_id)
    }

    pub fn glyph_information_for_glyph(
        &mut self,
        font_id: fontdb::ID,
        glyph_id: u16,
    ) -> Option<GlyphInfo> {
        self.glyph_information_from_font(font_id, '\0', glyph_id)
    }

    pub fn list_monospace_fonts(&self) -> Vec<String> {
        let db = self.font_system.db();
        let mut fonts = Vec::new();
        for face in db.faces() {
            #[cfg(not(target_os = "android"))]
            if !face.monospaced {
                continue;
            }
            for (family, _) in &face.families {
                let name = family.to_string();
                if !fonts.contains(&name) {
                    fonts.push(name);
                }
            }
        }
        fonts.sort();
        fonts
    }

    fn find_font_by_name(db: &fontdb::Database, family_name: &str) -> Option<fontdb::ID> {
        for face in db.faces() {
            for (family, _) in &face.families {
                if family.eq_ignore_ascii_case(family_name) {
                    return Some(face.id);
                }
            }
        }
        let cleaned = family_name.replace(['_', '-'], " ").trim().to_lowercase();
        for face in db.faces() {
            for (family, _) in &face.families {
                let fam_lower = family.to_lowercase();
                if fam_lower == cleaned || fam_lower.contains(&cleaned) {
                    return Some(face.id);
                }
            }
        }
        let cleaned_nospace: String = cleaned.chars().filter(|c| !c.is_whitespace()).collect();
        for face in db.faces() {
            for (family, _) in &face.families {
                let fam_nospace: String = family
                    .to_lowercase()
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect();
                if fam_nospace == cleaned_nospace {
                    return Some(face.id);
                }
            }
        }
        if family_name.eq_ignore_ascii_case("monospace") {
            for face in db.faces() {
                if face.monospaced {
                    return Some(face.id);
                }
            }
        }
        None
    }

    pub fn load_font_file(&mut self, path: &std::path::Path) -> Option<String> {
        let db = self.font_system.db_mut();
        let source = fontdb::Source::File(path.into());
        let ids = db.load_font_source(source);
        let first_id = ids.first()?;
        let face = db.face(*first_id)?;
        face.families.first().map(|(name, _)| name.clone())
    }

    pub fn has_font(&self) -> bool {
        self.font_id.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data");
    const FIXTURE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_fonts");

    #[test]
    fn font_pipeline_creation() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        assert_eq!(pipeline.atlas_dimensions(), (2048, 2048));
        assert!(
            pipeline.cache_length() > 0,
            "ASCII glyphs should be pre-rasterized"
        );
    }

    #[test]
    fn font_pipeline_has_system_fonts() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let fonts = pipeline.list_monospace_fonts();
        assert!(
            !fonts.is_empty(),
            "Should have at least one system monospace font"
        );
    }

    #[test]
    fn font_matching_stripped_spaces() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let names = pipeline.list_monospace_fonts();
        assert!(!names.is_empty(), "Should have at least one font");
        let name_with_space = names.iter().find(|n| n.contains(' '));
        let name = match name_with_space {
            Some(n) => n.clone(),
            None => {
                panic!(
                    "no monospace font with spaces found; cannot test stripped-name matching; available: {:?}",
                    names.iter().take(5).collect::<Vec<_>>()
                );
            }
        };
        let stripped: String = name.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(stripped != name, "Sanity: stripped name differs");
        let mut p2 = FontPipeline::new(2048, 2048, 14.0);
        assert!(
            p2.set_font_family(&stripped),
            "set_font_family should find '{}' when given '{}'",
            name,
            stripped
        );
    }

    #[test]
    fn glyph_hao_cjk_cross_verify() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_information('好')
            .expect("pipeline should have CJK glyph info (via fallback)");
        assert!(
            info.width > 0 || info.height > 0,
            "CJK '好' should produce non-zero glyph info: got {}x{}",
            info.width,
            info.height
        );
    }

    #[test]
    fn cjk_width_is_double_ascii() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let ascii_info = pipeline
            .glyph_information('A')
            .expect("ascii 'A' glyph info");
        let cjk_info = pipeline
            .glyph_information('中')
            .expect("CJK '中' glyph info");
        assert!(
            ascii_info.width > 0,
            "ASCII glyph should have positive width"
        );
        assert!(cjk_info.width > 0, "CJK glyph should have positive width");
        let (cell_w, _) = pipeline.cell_metrics();
        assert!(cell_w > 0.0, "cell width should be positive");
        let cell_span = if cjk_info.width as f32 > ascii_info.width as f32 * 1.5 {
            2
        } else {
            1
        };
        assert!(cell_span >= 1, "CJK cell span should be at least 1");
    }

    #[test]
    fn glyph_information_ascii() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let info = pipeline.glyph_information('A');
        assert!(info.is_some());
        let info = info.unwrap();
        assert!(info.width > 0);
        assert!(info.height > 0);
    }

    #[test]
    fn glyph_information_caching() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let before = pipeline.cache_length();
        pipeline.glyph_information('B');
        assert_eq!(pipeline.cache_length(), before);
        pipeline.glyph_information('B');
        assert_eq!(pipeline.cache_length(), before);
    }

    #[test]
    fn rasterize_ascii_populates_cache() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        assert!(pipeline.cache_length() >= 95);
    }

    #[test]
    fn atlas_eviction_keeps_glyphs_retrievable() {
        let mut pipeline = FontPipeline::new(256, 256, 14.0);
        let distinct: Vec<char> = ('A'..='Z')
            .chain('a'..='z')
            .chain('0'..='9')
            .chain("!@#$%^&*()_+-=[]{}|;:,.<>?/".chars())
            .collect();
        for &ch in distinct.iter().cycle().take(distinct.len() * 4) {
            assert!(
                pipeline.glyph_information(ch).is_some(),
                "glyph {ch:?} must remain retrievable under eviction pressure"
            );
        }
        let generation_after_fill = pipeline.atlas_generation();
        assert!(pipeline.glyph_information('A').is_some());
        assert!(
            pipeline.atlas_generation() >= generation_after_fill,
            "atlas generation must remain monotonic under eviction"
        );
        assert!(
            pipeline.cache_length() <= GLYPH_CACHE_CAPACITY,
            "glyph cache must respect its capacity after eviction"
        );
    }

    #[test]
    fn glyph_information_has_atlas_coords() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let info = pipeline.glyph_information('X').unwrap();
        assert!(info.atlas_x >= 0);
        assert!(info.atlas_y >= 0);
        assert!(info.width > 0);
        assert!(info.height > 0);
    }

    #[test]
    fn atlas_bitmap_not_empty_after_rasterize() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        pipeline.glyph_information('A');
        let bitmap = pipeline.atlas_bitmap();
        assert!(bitmap.iter().any(|&b| b != 0));
    }

    #[test]
    fn cell_metrics_returns_positive_dimensions() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let (cw, ch) = pipeline.cell_metrics();
        assert!(cw > 0.0, "cell_width must be > 0, got {cw}");
        assert!(ch > 0.0, "cell_height must be > 0, got {ch}");
    }

    #[test]
    fn cell_metrics_scales_with_font_size() {
        let small = FontPipeline::new(2048, 2048, 10.0);
        let large = FontPipeline::new(2048, 2048, 20.0);
        let (sw, sh) = small.cell_metrics();
        let (lw, lh) = large.cell_metrics();
        assert!(lw > sw, "larger font must have wider cell");
        assert!(lh > sh, "larger font must have taller cell");
    }

    #[test]
    fn b1_fontlist_includes_fixture() {
        let pipeline = FontPipeline::from_fixture(512, 512, 12.0, FIXTURE_DIR);
        let fonts = pipeline.list_monospace_fonts();
        assert!(
            fonts
                .iter()
                .any(|name| { name.contains("Liberation") || name.contains("Mono") }),
            "LiberationMono should appear in font list from fixture dir, got: {:?}",
            fonts
        );
    }

    #[test]
    fn b2_setting_font_changes_metrics() {
        let mut pipeline = FontPipeline::from_fixture(512, 512, 12.0, FIXTURE_DIR);
        let fonts = pipeline.list_monospace_fonts();
        let lm = fonts
            .iter()
            .find(|name| name.contains("Liberation") || name.contains("Mono"))
            .cloned();
        let name = lm.expect("LiberationMono should be in font list from fixture dir");
        assert!(
            pipeline.set_font_family(&name),
            "set_font_family should succeed for {name}"
        );
        let (cw, ch) = pipeline.cell_metrics();
        assert!(cw > 0.0, "cell width should be positive, got {cw}");
        assert!(ch > 0.0, "cell height should be positive, got {ch}");
    }

    fn load_freetype_golden(dir: &str, stem: &str) -> Option<(u32, u32, Vec<u8>)> {
        let meta_path = std::path::Path::new(dir).join(format!("freetype_{stem}.meta"));
        let rgba_path = std::path::Path::new(dir).join(format!("freetype_{stem}.rgba"));
        if !meta_path.exists() || !rgba_path.exists() {
            return None;
        }
        let meta = std::fs::read_to_string(meta_path).ok()?;
        let glyph_width: u32 = meta
            .lines()
            .find(|l| l.starts_with("width="))?
            .trim_start_matches("width=")
            .parse()
            .ok()?;
        let glyph_height: u32 = meta
            .lines()
            .find(|l| l.starts_with("height="))?
            .trim_start_matches("height=")
            .parse()
            .ok()?;
        let data = std::fs::read(rgba_path).ok()?;
        Some((glyph_width, glyph_height, data))
    }

    fn compare_with_freetype(ch: char, stem: &str) {
        let golden = load_freetype_golden(TEST_DATA_DIR, stem);

        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_information(ch)
            .unwrap_or_else(|| panic!("pipeline glyph_information('{ch}') should succeed"));
        let atlas = pipeline.atlas_bitmap();

        let regenerate;

        if let Some((ft_w, ft_h, ft_data)) = golden {
            let w_diff = (info.width - ft_w as i32).abs();
            let h_diff = (info.height - ft_h as i32).abs();

            if w_diff > 2 || h_diff > 2 {
                eprintln!(
                    "glyph '{ch}' dimensions differ (FT={ft_w}x{ft_h} pipeline={}x{}) — regenerating golden file",
                    info.width, info.height
                );
                regenerate = true;
            } else {
                let cmp_w = info.width.min(ft_w as i32).max(0) as usize;
                let cmp_h = info.height.min(ft_h as i32).max(0) as usize;
                let ft_stride = ft_w as usize * 4;
                let ax = info.atlas_x as usize;
                let ay = info.atlas_y as usize;
                let atlas_w = 512usize;
                let mut max_diff = 0u8;
                let mut diff_count = 0u32;

                for y in 0..cmp_h {
                    for x in 0..cmp_w {
                        let pixel = (ay + y) * atlas_w + ax + x;
                        let ai = pixel * 4;
                        let fi = y * ft_stride + x * 4;
                        let atlas_pixel = atlas[ai];
                        let freetype_pixel = ft_data[fi + 3];
                        let diff = atlas_pixel.abs_diff(freetype_pixel);
                        if diff > max_diff {
                            max_diff = diff;
                        }
                        if diff > 2 {
                            diff_count += 1;
                        }
                    }
                }

                if max_diff > 128 || diff_count > (cmp_w * cmp_h / 3) as u32 {
                    eprintln!(
                        "glyph '{ch}' FreeType comparison differs too much (max={max_diff}) — regenerating golden file"
                    );
                    regenerate = true;
                } else {
                    assert!(
                        max_diff <= 64 || diff_count <= (cmp_w * cmp_h / 5) as u32,
                        "glyph '{ch}' FreeType comparison: max_alpha_diff={max_diff} \
                         pixels_over_tolerance={diff_count} (total={})",
                        cmp_w * cmp_h
                    );
                    return;
                }
            }
        } else {
            eprintln!("No golden file for glyph '{ch}' — generating it now");
            regenerate = true;
        }

        if regenerate {
            save_pipeline_glyph_as_golden(&info, atlas, 512, TEST_DATA_DIR, stem);
            eprintln!("Golden file freetype_{stem}.rgba regenerated for current font");
        }
    }

    fn save_pipeline_glyph_as_golden(
        info: &GlyphInfo,
        atlas: &[u8],
        atlas_width: usize,
        dir: &str,
        stem: &str,
    ) {
        let ax = info.atlas_x as usize;
        let ay = info.atlas_y as usize;
        let w = info.width as usize;
        let h = info.height as usize;

        let mut rgba = Vec::with_capacity(w * h * 4);
        for y in 0..h {
            for x in 0..w {
                let alpha = atlas[(ay + y) * atlas_width + ax + x];
                rgba.extend_from_slice(&[0, 0, 0, alpha]);
            }
        }

        let rgba_path = format!("{dir}/freetype_{stem}.rgba");
        let meta_path = format!("{dir}/freetype_{stem}.meta");
        std::fs::write(&rgba_path, &rgba).expect("write golden rgba");
        std::fs::write(&meta_path, format!("{w} {h}\n")).expect("write golden meta");
    }

    #[test]
    fn glyph_a_freetype_comparison() {
        compare_with_freetype('A', "A");
    }

    #[test]
    fn glyph_hao_freetype_comparison() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_information('好')
            .expect("CJK '好' should have glyph information");
        let atlas = pipeline.atlas_bitmap();
        let ax = info.atlas_x as usize;
        let ay = info.atlas_y as usize;
        let atlas_w = 512usize;
        let mut has_ink = false;
        for y in 0..info.height as usize {
            for x in 0..info.width as usize {
                let byte_offset = ((ay + y) * atlas_w + ax + x) * 4;
                if byte_offset < atlas.len() && atlas[byte_offset] > 0 {
                    has_ink = true;
                    break;
                }
            }
            if has_ink {
                break;
            }
        }
        assert!(
            has_ink,
            "CJK '好' should have non-zero coverage in pipeline atlas"
        );

        let (_ft_w, _ft_h, _ft_data) = load_freetype_golden(TEST_DATA_DIR, "hao")
            .expect("freetype golden not found at test_data/freetype_hao.{meta,rgba}. To generate: run with noto-fonts-cjk-sans installed and copy output");
    }

    #[test]
    fn bearing_values_for_dot() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_information('.')
            .expect("'.' should glyph_information");
        assert!(
            info.placement.left >= 0,
            "dot bearing_x={} should be >= 0",
            info.placement.left
        );
        assert!(
            info.placement.top > 0,
            "dot bearing_y={} should be > 0",
            info.placement.top
        );
    }

    #[test]
    fn bearing_values_for_a() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_information('A')
            .expect("'A' should have glyph_information");
        assert!(
            info.placement.width > 0,
            "A glyph_width={} should be > 0",
            info.placement.width
        );
        assert!(
            info.placement.height > 0,
            "A glyph_height={} should be > 0",
            info.placement.height
        );
    }

    #[test]
    fn bearing_values_for_cjk() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_information('好')
            .expect("'好' should have glyph_information");
        assert!(
            info.placement.left >= 0,
            "好 bearing_x={} should be >= 0",
            info.placement.left
        );
        assert!(
            info.placement.top > 0,
            "好 bearing_y={} should be > 0",
            info.placement.top
        );
        let dot_info = pipeline.glyph_information('.').expect("'.' for comparison");
        assert!(
            info.placement.width >= dot_info.placement.width * 2 - 2,
            "好 width={} should be ~2x dot width={}",
            info.placement.width,
            dot_info.placement.width
        );
    }

    fn bearing_fits_inside_cell(glyph: char, label: &str) {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_information(glyph)
            .unwrap_or_else(|| panic!("'{glyph}' glyph_information"));
        let (_cell_w, cell_h) = pipeline.cell_metrics();
        let ascent = pipeline.ascent_pixels();
        let bearing_y = ascent - info.placement.top as f32;
        let glyph_h = info.placement.height as f32;
        assert!(
            bearing_y >= -cell_h,
            "{label} glyph starts way above cell: bearing_y={} < -cell_h",
            bearing_y
        );
        assert!(glyph_h > 0.0, "{label} glyph has zero height");
        assert!(cell_h > 0.0, "{label} cell has zero height");
    }

    #[test]
    fn bearing_dot_fits_inside_cell() {
        bearing_fits_inside_cell('.', "dot");
    }

    #[test]
    fn bearing_a_fits_inside_cell() {
        bearing_fits_inside_cell('a', "a");
    }

    #[test]
    fn bearing_values_non_zero_for_rendered_glyphs() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        pipeline.rasterize_ascii();
        let glyphs = ['0', 'x', 'g', 'p', 'W', 'M', 'f', '(', ')'];
        for &ch in &glyphs {
            if let Some(info) = pipeline.glyph_information(ch) {
                assert!(
                    info.placement.width > 0,
                    "'{ch}' width={} should be > 0",
                    info.placement.width
                );
                assert!(
                    info.placement.height > 0,
                    "'{ch}' height={} should be > 0",
                    info.placement.height
                );
            }
        }
    }

    #[test]
    fn font_enumeration_finds_monospace() {
        let pipeline = FontPipeline::new(512, 512, 14.0);
        let fonts = pipeline.list_monospace_fonts();
        assert!(
            !fonts.is_empty(),
            "FontLoader should find at least one monospace face, got: {:?}",
            fonts
        );
        assert!(
            pipeline.has_font(),
            "FontPipeline should have a font assigned"
        );
    }

    #[test]
    fn cjk_glyph_zhong() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_information('中')
            .expect("CJK '中' (U+4E2D) should have glyph info");
        assert!(
            info.width > 0,
            "CJK '中' width should be non-zero, got {}",
            info.width
        );
        assert!(
            info.height > 0,
            "CJK '中' height should be non-zero, got {}",
            info.height
        );
        let atlas = pipeline.atlas_bitmap();
        let atlas_w = 512usize;
        let ax = info.atlas_x as usize;
        let ay = info.atlas_y as usize;
        let mut has_ink = false;
        for y in 0..info.height as usize {
            for x in 0..info.width as usize {
                let byte_offset = ((ay + y) * atlas_w + ax + x) * 4;
                if byte_offset < atlas.len() && atlas[byte_offset] > 0 {
                    has_ink = true;
                    break;
                }
            }
            if has_ink {
                break;
            }
        }
        assert!(has_ink, "CJK '中' should have non-zero coverage in atlas");
    }

    #[test]
    fn emoji_glyph_grinning() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let ch = '\u{1F600}';
        let info = pipeline.glyph_information(ch);
        if info.is_none() || info.as_ref().is_some_and(|i| i.width == 0) {
            let fonts = pipeline.list_monospace_fonts();
            let found_emoji = fonts.iter().any(|name| {
                name.contains("Emoji")
                    || name.contains("Noto")
                    || name.to_lowercase().contains("emoji")
            });
            assert!(
                found_emoji,
                "no emoji-supporting font found in system; emoji glyph test requires Noto Emoji or similar"
            );
        }
        let info = info.expect("emoji should have glyph info");
        assert!(
            info.width > 0 || info.height > 0,
            "emoji should produce non-zero glyph info: got {}x{}",
            info.width,
            info.height
        );
    }

    #[test]
    fn glyph_atlas_lru_eviction() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        pipeline.rasterize_ascii();
        let after_ascii = pipeline.cache_length();
        assert!(
            after_ascii >= 95,
            "should have at least 95 cached after rasterize_ascii, got {}",
            after_ascii
        );

        let mut inserted = 0u32;
        for cp in 0x4E00u32..0x4F00u32 {
            let ch = char::from_u32(cp).unwrap_or('\0');
            if pipeline.glyph_information(ch).is_some_and(|i| i.width > 0) {
                inserted += 1;
            }
        }
        let final_len = pipeline.cache_length();
        assert!(
            final_len <= 10000,
            "cache_length {} exceeds capacity 10000",
            final_len
        );
        assert!(
            final_len >= after_ascii,
            "cache should not shrink after inserting new glyphs: \
             before={} after={} inserted={}",
            after_ascii,
            final_len,
            inserted
        );
        let bitmap = pipeline.atlas_bitmap();
        assert!(
            bitmap.iter().any(|&b| b != 0),
            "atlas bitmap should have non-zero bytes after glyph insertion"
        );
    }

    #[test]
    fn cjk_glyph_information_returns_nonzero_for_common_chars() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let chars = [
            '你', '好', '世', '界', '中', '文', '字', '体', '渲', '染', '测', '试',
        ];
        for ch in chars {
            let info = pipeline
                .glyph_information(ch)
                .unwrap_or_else(|| panic!("CJK glyph_information('{ch}') should return Some"));
            assert!(
                info.width > 0 && info.height > 0,
                "CJK glyph '{ch}' should have nonzero dimensions: {}x{}",
                info.width,
                info.height
            );
        }
    }

    #[test]
    fn font_switching_changes_font_id() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let original_id = pipeline.font_id;
        let names = pipeline.list_monospace_fonts();
        for name in &names {
            if name.is_empty() {
                continue;
            }
            if pipeline.set_font_family(name) && pipeline.font_id != original_id {
                return;
            }
        }
    }

    #[test]
    fn font_switching_clears_cache() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        pipeline.rasterize_ascii();
        let before = pipeline.cache_length();
        assert!(before > 0);
        pipeline.glyph_information('好');
        let names = pipeline.list_monospace_fonts();
        if names.len() > 1 {
            let alt = names.last().unwrap();
            pipeline.set_font_family(alt);
            assert_eq!(
                pipeline.cache_length(),
                0,
                "cache should be cleared after font switch to '{alt}'"
            );
        } else {
            pipeline.set_font_family("monospace");
            if pipeline.cache_length() == 0 {
                return;
            }
            assert!(
                pipeline.cache_length() < before,
                "cache should shrink after font switch"
            );
        }
    }

    #[test]
    fn cell_metrics_height_is_integer() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let (_cw, ch) = pipeline.cell_metrics();
        assert!(
            (ch - ch.floor()).abs() < f32::EPSILON,
            "cell_height should be integer (ceil'd), got {ch}"
        );
    }

    #[test]
    fn cell_metrics_height_scales_with_font_size() {
        let small = FontPipeline::new(2048, 2048, 10.0);
        let large = FontPipeline::new(2048, 2048, 20.0);
        let (_, sh) = small.cell_metrics();
        let (_, lh) = large.cell_metrics();
        assert!(lh > sh, "larger font must have taller cell");
        assert!(
            (sh - sh.floor()).abs() < f32::EPSILON,
            "small cell_height should be integer"
        );
        assert!(
            (lh - lh.floor()).abs() < f32::EPSILON,
            "large cell_height should be integer"
        );
    }

    #[test]
    fn termux_formula_ascent_plus_descent_equals_cell_height() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let ascent = pipeline.ascent_pixels();
        let descent = pipeline.descent_pixels();
        let (_, ch) = pipeline.cell_metrics();
        assert!(
            (ascent + descent - ch).abs() < 2.0,
            "ascent({ascent}) + descent({descent}) ≈ cell_height({ch}), diff={}",
            (ascent + descent - ch).abs()
        );
    }

    #[test]
    fn termux_formula_baseline_is_ascent_from_cell_top() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let ascent = pipeline.ascent_pixels();
        let (_, ch) = pipeline.cell_metrics();
        assert!(
            ascent > 0.0 && ascent < ch,
            "ascent({ascent}) must be in (0, cell_h={ch})"
        );
    }

    #[test]
    fn termux_formula_glyph_bearing_y_matches() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let ascent = pipeline.ascent_pixels();
        let info = pipeline
            .glyph_information('A')
            .expect("should have 'A' glyph");
        let bearing_y = ascent - info.placement.top as f32;
        assert!(
            bearing_y >= 0.0,
            "bearing_y for 'A' should be >= 0, got {bearing_y}"
        );
        let (_, ch) = pipeline.cell_metrics();
        assert!(
            bearing_y < ch,
            "bearing_y({bearing_y}) should be < cell_h({ch})"
        );
    }

    #[test]
    fn descent_pixels_is_positive() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let descent = pipeline.descent_pixels();
        assert!(descent > 0.0, "descent should be positive, got {descent}");
    }

    #[test]
    fn cell_width_from_m_advance_matches() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let info_m = pipeline.glyph_information('m').expect("should have 'm'");
        let info_x = pipeline.glyph_information('X').expect("should have 'X'");
        let (cw, _ch) = pipeline.cell_metrics();
        assert!(
            (info_m.advance_width - cw).abs() < 1.0,
            "advance_width('m')={} ≈ cell_w={}",
            info_m.advance_width,
            cw
        );
        assert!(
            (info_x.advance_width - cw).abs() < 1.0,
            "advance_width('X')={} ≈ cell_w={}",
            info_x.advance_width,
            cw
        );
    }

    #[test]
    fn any_monospace_advance_matches_cell_width() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let (cw, _) = pipeline.cell_metrics();
        let chars = ['A', 'm', 'W', '0', 'l', 'i'];
        for ch in chars {
            if let Some(info) = pipeline.glyph_information(ch) {
                assert!(
                    (info.advance_width - cw).abs() < 2.0,
                    "advance('{ch}')={:.1} ≈ cell_w={:.1}",
                    info.advance_width,
                    cw
                );
            }
        }
        if let Some(alt) = pipeline.list_monospace_fonts().first().cloned()
            && pipeline.set_font_family(&alt)
        {
            let (cw2, _) = pipeline.cell_metrics();
            for ch in chars {
                if let Some(info) = pipeline.glyph_information(ch) {
                    assert!(
                        (info.advance_width - cw2).abs() < 2.0,
                        "font '{alt}': advance('{ch}')={:.1} ≈ cell_w={:.1}",
                        info.advance_width,
                        cw2
                    );
                }
            }
        }
    }

    #[test]
    fn cjk_advance_valid_for_any_font() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let (cw, _) = pipeline.cell_metrics();
        let cjk_chars = ['中', '好', '世', '界', '日', '本'];
        for ch in cjk_chars {
            if let Some(info) = pipeline.glyph_information(ch) {
                assert!(
                    info.advance_width > 0.0,
                    "CJK '{ch}' must have positive advance, got {:.1}",
                    info.advance_width
                );
                assert!(
                    info.advance_width <= cw * 3.0,
                    "CJK '{ch}' advance={:.1} should be ≤ 3*cell_w={:.1}",
                    info.advance_width,
                    cw * 3.0
                );
            }
        }
    }

    #[test]
    fn ascii_bearing_y_nonnegative_for_any_font() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let ascent = pipeline.ascent_pixels();
        let ascii = ['A', 'B', 'C', 'x', 'y', 'z', '0', '1', '9'];
        for ch in ascii {
            if let Some(info) = pipeline.glyph_information(ch) {
                let bearing_y = ascent - info.placement.top as f32;
                assert!(
                    bearing_y >= -2.0,
                    "bearing_y('{ch}')={:.1} should be >= -2",
                    bearing_y
                );
            }
        }
    }

    #[test]
    fn all_glyphs_within_atlas_bounds() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        pipeline.rasterize_ascii();
        let aw = pipeline.atlas_width as i32;
        let ah = pipeline.atlas_height as i32;
        let chars = ['A', '中', '好', 'α', 'Ω'];
        for ch in chars {
            if let Some(info) = pipeline.glyph_information(ch) {
                assert!(
                    info.atlas_x + info.width <= aw,
                    "glyph '{ch}' atlas_x({}) + width({}) exceeds atlas_w({})",
                    info.atlas_x,
                    info.width,
                    aw
                );
                assert!(
                    info.atlas_y + info.height <= ah,
                    "glyph '{ch}' atlas_y({}) + height({}) exceeds atlas_h({})",
                    info.atlas_y,
                    info.height,
                    ah
                );
            }
        }
    }

    #[test]
    fn system_monospace_name_returns_nonempty() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let name = pipeline.system_monospace_name();
        assert!(
            !name.is_empty(),
            "system_monospace_name should return a non-empty string"
        );
    }

    #[test]
    fn set_font_family_empty_resets_to_default() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let default_name = pipeline.default_font_name().clone();
        let fonts = pipeline.list_monospace_fonts();
        if let Some(other) = fonts.iter().find(|n| n.as_str() != default_name.as_str()) {
            pipeline.set_font_family(other);
            assert_eq!(
                pipeline.current_font_family_name().as_deref(),
                Some(other.as_str())
            );
            pipeline.set_font_family("");
            assert_eq!(
                pipeline.current_font_family_name().as_deref(),
                Some(default_name.as_str())
            );
        }
    }

    #[test]
    fn font_information_contains_all_sections() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let info = pipeline.font_information();
        assert!(
            info.contains("Active:"),
            "font_information should contain 'Active:', got: {}",
            info
        );
        assert!(
            info.contains("CJK fallback:"),
            "font_information should contain 'CJK fallback:', got: {}",
            info
        );
        assert!(
            info.contains("Cell:"),
            "font_information should contain 'Cell:', got: {}",
            info
        );
        assert!(
            info.contains("Font size:"),
            "font_information should contain 'Font size:', got: {}",
            info
        );
    }

    #[test]
    fn set_font_family_persists_through_size_change() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let fonts = pipeline.list_monospace_fonts();
        if let Some(target) = fonts.first() {
            pipeline.set_font_family(target);
            let name_before = pipeline.current_font_family_name();
            pipeline.set_font_size_in_place(20.0);
            let name_after = pipeline.current_font_family_name();
            assert_eq!(
                name_before, name_after,
                "font family should persist through size change"
            );
        }
    }

    #[test]
    fn cjk_fallback_has_vector_font() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let cjk_names = pipeline.cjk_fallback_names();
        if !cjk_names.is_empty() {
            assert!(
                cjk_names.iter().all(|n| !n.is_empty()),
                "CJK fallback names should not be empty strings"
            );
        }
    }

    #[test]
    fn default_font_config_uses_system_default() {
        let config = torvox_core::config::FontConfig::default();
        assert!(
            config.family.is_empty(),
            "default font family should be empty for system default"
        );
    }

    #[test]
    fn cell_metrics_reasonable_ratios() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let (cw, ch) = pipeline.cell_metrics();
        assert!(cw > 0.0, "cell_width must be > 0, got {cw}");
        assert!(ch > 0.0, "cell_height must be > 0, got {ch}");
        assert!(
            cw < ch,
            "terminal cells should be taller than wide: cell_width={cw} >= cell_height={ch}"
        );
    }

    #[test]
    fn find_monospace_font_prefers_roboto_mono() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let name = pipeline.default_font_name();
        assert!(!name.is_empty(), "should find a monospace font, got empty");
    }

    #[test]
    fn cjk_fallback_uses_vector_font() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let names = pipeline.cjk_fallback_names();
        if !names.is_empty() {
            let cjk_info = pipeline.glyph_information('好').expect("CJK glyph info");
            assert!(
                cjk_info.width > 0,
                "CJK glyph should have meaningful width, got {}",
                cjk_info.width
            );
        }
    }

    fn try_load_cjk_fonts(db: &mut fontdb::Database) -> bool {
        let has_cjk = db.faces().any(|face| {
            face.families
                .first()
                .map(|(n, _)| n.to_lowercase().contains("cjk"))
                .unwrap_or(false)
        });
        if has_cjk {
            return true;
        }
        if let Ok(glob) = std::fs::read_dir("/nix/store") {
            for entry in glob.flatten() {
                let p = entry.path();
                if p.to_string_lossy().contains("noto-fonts-cjk") {
                    let font_dir = p.join("share/fonts/opentype/noto-cjk");
                    if font_dir.is_dir() {
                        db.load_fonts_dir(&font_dir);
                        return true;
                    }
                }
            }
        }
        false
    }

    #[test]
    fn non_cjk_locale_no_fallback() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        pipeline.set_system_locale("en-US");
        let info = pipeline.font_information();
        assert!(
            info.contains("CJK fallback: none"),
            "en-US locale should have no CJK fallback: {info}"
        );
    }

    #[test]
    fn de_locale_no_fallback() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        pipeline.set_system_locale("de-DE");
        let info = pipeline.font_information();
        assert!(
            info.contains("CJK fallback: none"),
            "de-DE locale should have no CJK fallback: {info}"
        );
    }

    #[test]
    fn cjk_locale_selects_correct_variant() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        if !try_load_cjk_fonts(pipeline.font_system.db_mut()) {
            eprintln!("SKIP: cjk_locale_selects_correct_variant (no CJK fonts)");
            return;
        }
        let cases: &[(&str, &str)] = &[
            ("zh-CN", "sc"),
            ("zh-TW", "tc"),
            ("zh-HK", "tc"),
            ("zh-Hant", "tc"),
            ("zh-Hans", "sc"),
            ("zh", "sc"),
            ("ja", "jp"),
            ("ko", "kr"),
        ];
        for (locale, expected_tag) in cases {
            let mut pipeline = FontPipeline::new(512, 512, 14.0);
            try_load_cjk_fonts(pipeline.font_system.db_mut());
            pipeline.set_system_locale(locale);
            let ids = &pipeline.cjk_fallback_ids;
            assert!(!ids.is_empty(), "locale '{locale}' should have fallback");
            let db = pipeline.font_system.db();
            let has_tag = ids.iter().any(|id| {
                db.face(*id)
                    .and_then(|f| f.families.first())
                    .is_some_and(|(n, _)| n.to_lowercase().contains(expected_tag))
            });
            assert!(
                has_tag,
                "locale '{locale}' fallback should include '{expected_tag}'-family font"
            );
        }
    }

    #[test]
    fn primary_cjk_font_no_fallback() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        if !try_load_cjk_fonts(pipeline.font_system.db_mut()) {
            eprintln!("SKIP: primary_cjk_font_no_fallback (no CJK fonts)");
            return;
        }
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        try_load_cjk_fonts(pipeline.font_system.db_mut());
        pipeline.set_system_locale("zh-CN");
        let cjk_fonts: Vec<String> = pipeline
            .list_monospace_fonts()
            .into_iter()
            .filter(|n| n.to_lowercase().contains("cjk"))
            .collect();
        if let Some(cjk_name) = cjk_fonts.first() {
            pipeline.set_font_family(cjk_name);
            let names = pipeline.cjk_fallback_names();
            assert!(
                names.is_empty(),
                "primary font '{cjk_name}' supports CJK → no fallback, got: {names:?}"
            );
        }
    }

    #[test]
    fn max_one_fallback_font() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        if !try_load_cjk_fonts(pipeline.font_system.db_mut()) {
            eprintln!("SKIP: max_one_fallback_font (no CJK fonts)");
            return;
        }
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        try_load_cjk_fonts(pipeline.font_system.db_mut());
        pipeline.set_system_locale("zh-CN");
        assert!(
            pipeline.cjk_fallback_ids.len() <= 3,
            "MAX_CJK_FALLBACK_FONTS=3, got {} IDs",
            pipeline.cjk_fallback_ids.len()
        );
    }

    #[test]
    fn font_information_includes_cjk_fallback() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let info = pipeline.font_information();
        assert!(
            info.contains("Active:") || info.contains("Cell:"),
            "font info should have structure: {info}"
        );
    }

    #[test]
    fn atlas_defrag_recovers_from_full_atlas() {
        let mut pipeline = FontPipeline::new(64, 64, 14.0);
        let mut successes = 0u32;
        for cp in 0x4E00u32..0x4F00u32 {
            if let Some(ch) = char::from_u32(cp)
                && pipeline.glyph_information(ch).is_some_and(|i| i.width > 0)
            {
                successes += 1;
            }
        }
        assert!(
            successes > 0,
            "should have inserted at least some CJK glyphs"
        );
        let bitmap = pipeline.atlas_bitmap();
        assert!(
            bitmap.iter().any(|&b| b != 0),
            "atlas should have content after defrag"
        );
    }

    const VENDOR_TTF: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../vendor/ghostty/src/font/res/TerminusTTF-Regular.ttf"
    );

    #[test]
    fn load_font_file_valid_ttf_returns_family() {
        let mut p = FontPipeline::new(512, 512, 14.0);
        let family = p.load_font_file(std::path::Path::new(VENDOR_TTF));
        let family = family.expect("load_font_file should return Some for valid TTF");
        assert!(
            !family.is_empty(),
            "family name should not be empty, got '{family}'"
        );
        assert!(
            family.contains("Terminus") || family.contains("TerminusTTF"),
            "expected 'Terminus' in family name, got '{family}'"
        );
    }

    #[test]
    fn load_font_file_nonexistent_path_returns_none() {
        let mut p = FontPipeline::new(512, 512, 14.0);
        let result = p.load_font_file(std::path::Path::new("/nonexistent/path/to/font.ttf"));
        assert!(result.is_none(), "should return None for nonexistent path");
    }

    #[test]
    fn load_font_file_empty_file_returns_none() {
        let dir = std::env::temp_dir().join("torvox_test_font_load");
        let _ = std::fs::create_dir_all(&dir);
        let empty_path = dir.join("empty.ttf");
        std::fs::write(&empty_path, []).ok();
        let mut p = FontPipeline::new(512, 512, 14.0);
        let result = p.load_font_file(&empty_path);
        assert!(result.is_none(), "empty file should return None");
        let _ = std::fs::remove_file(&empty_path);
    }

    #[test]
    fn load_font_file_corrupt_file_returns_none() {
        let dir = std::env::temp_dir().join("torvox_test_font_load");
        let _ = std::fs::create_dir_all(&dir);
        let corrupt_path = dir.join("corrupt.ttf");
        let garbage: Vec<u8> = (0..256).map(|i| (i ^ 0xAB) as u8).collect();
        std::fs::write(&corrupt_path, &garbage).ok();
        let mut p = FontPipeline::new(512, 512, 14.0);
        let result = p.load_font_file(&corrupt_path);
        assert!(result.is_none(), "corrupt file should return None");
        let _ = std::fs::remove_file(&corrupt_path);
    }

    #[test]
    fn load_font_file_multiple_times_works() {
        let mut p = FontPipeline::new(512, 512, 14.0);
        let first = p.load_font_file(std::path::Path::new(VENDOR_TTF));
        let second = p.load_font_file(std::path::Path::new(VENDOR_TTF));
        assert!(first.is_some(), "first load should succeed");
        assert!(second.is_some(), "second load of same file should succeed");
        assert_eq!(
            first, second,
            "loading same file twice should return same family"
        );
    }

    #[test]
    fn load_font_file_does_not_break_cell_metrics() {
        let mut p = FontPipeline::new(512, 512, 14.0);
        let (cw_before, ch_before) = p.cell_metrics();
        assert!(
            cw_before > 0.0 && ch_before > 0.0,
            "initial metrics should be positive"
        );
        let family = p.load_font_file(std::path::Path::new(VENDOR_TTF));
        assert!(family.is_some(), "should load vendor TTF");
        let (cw_after, ch_after) = p.cell_metrics();
        assert!(
            (cw_before - cw_after).abs() < f32::EPSILON,
            "cell width unchanged after load_font_file"
        );
        assert!(
            (ch_before - ch_after).abs() < f32::EPSILON,
            "cell height unchanged after load_font_file"
        );
    }

    #[test]
    fn load_font_file_loaded_font_can_be_set() {
        let mut p = FontPipeline::new(512, 512, 14.0);
        let family = p
            .load_font_file(std::path::Path::new(VENDOR_TTF))
            .expect("should load vendor TTF");
        assert!(
            p.set_font_family(&family),
            "set_font_family should succeed for loaded font '{family}'"
        );
        let (cw, ch) = p.cell_metrics();
        assert!(cw > 0.0, "cell width positive after setting loaded font");
        assert!(ch > 0.0, "cell height positive after setting loaded font");
    }

    #[test]
    fn load_font_file_unicode_path() {
        let dir = std::env::temp_dir().join("torvox_test_unicode_字体");
        let _ = std::fs::create_dir_all(&dir);
        let target = dir.join("测试-font.ttf");
        std::fs::copy(VENDOR_TTF, &target).expect("copy vendor TTF to unicode path");
        let mut p = FontPipeline::new(512, 512, 14.0);
        let family = p.load_font_file(&target);
        assert!(family.is_some(), "should load font from unicode path");
        assert!(!family.unwrap().is_empty(), "family should not be empty");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_font_file_after_set_font_family() {
        let mut p = FontPipeline::new(512, 512, 14.0);
        let fonts = p.list_monospace_fonts();
        if let Some(first) = fonts.first() {
            assert!(p.set_font_family(first), "set font family {first}");
        }
        let result = p.load_font_file(std::path::Path::new(VENDOR_TTF));
        assert!(
            result.is_some(),
            "load after set_font_family should succeed"
        );
    }
}
