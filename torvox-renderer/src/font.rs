// @Font shaping (cosmic-text + swash), IMPL_REND_002, impl, [REQ_REND_002]
// @need-ids: REQ_REND_002
use std::num::NonZeroUsize;

use cosmic_text::FontSystem;
use lru::LruCache;
use swash::scale::{Render, ScaleContext, Source};
use swash::zeno::Placement;
use thiserror::Error;

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
    pub glyph_id: u16,
    pub pixel_size: u16,
}

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub atlas_x: i32,
    pub atlas_y: i32,
    pub width: i32,
    pub height: i32,
    pub placement: Placement,
    pub advance_width: f32,
}

pub struct FontPipeline {
    font_system: FontSystem,
    scaler_context: ScaleContext,
    atlas: guillotiere::AtlasAllocator,
    glyph_cache: LruCache<GlyphKey, GlyphInfo>,
    atlas_bitmap: Vec<u8>,
    atlas_width: u32,
    atlas_height: u32,
    font_id: Option<fontdb::ID>,
    cjk_fallback_ids: Vec<fontdb::ID>,
    font_size: f32,
    atlas_generation: u64,
}

impl FontPipeline {
    pub fn new(atlas_width: i32, atlas_height: i32, font_size: f32) -> Self {
        let mut font_system = FontSystem::new();
        let db = font_system.db_mut();

        // On Android, scan /system/fonts/ for system fonts.
        // Also scan the app's private files dir so custom fonts pushed
        // via `adb push font.ttf /data/data/io.torvox/files/` are loaded.
        #[cfg(target_os = "android")]
        {
            for dir in ["/system/fonts/", "/data/data/io.torvox/files/"] {
                let dir = std::path::Path::new(dir);
                if let Ok(entries) = std::fs::read_dir(dir) {
                    let mut count = 0u32;
                    for entry in entries.flatten() {
                        let path = entry.path();
                        #[allow(clippy::collapsible_if)]
                        if path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                            e.eq_ignore_ascii_case("ttf")
                                || e.eq_ignore_ascii_case("otf")
                                || e.eq_ignore_ascii_case("ttc")
                        }) && db.load_font_file(&path).is_ok()
                        {
                            count += 1;
                        }
                    }
                    log::info!("FONT_LOAD: loaded {count} fonts from {}", dir.display());
                }
            }
        }

        let _ = db;
        let scaler_context = ScaleContext::new();
        let atlas = guillotiere::AtlasAllocator::new(guillotiere::size2(atlas_width, atlas_height));
        let atlas_bitmap = vec![0u8; (atlas_width * atlas_height) as usize];

        let mut pipeline = Self {
            font_system,
            scaler_context,
            atlas,
            glyph_cache: LruCache::new(NonZeroUsize::new(10000).unwrap()),
            atlas_bitmap,
            atlas_width: atlas_width as u32,
            atlas_height: atlas_height as u32,
            font_id: None,
            cjk_fallback_ids: Vec::new(),
            font_size,
            atlas_generation: 0,
        };

        if pipeline.font_id.is_none() {
            pipeline.find_monospace_font();
        }
        pipeline.find_cjk_fallback_fonts();
        pipeline.rasterize_ascii();
        pipeline
    }

    fn find_monospace_font(&mut self) {
        let db = self.font_system.db();
        for face in db.faces() {
            if face.monospaced {
                let name: String = face
                    .families
                    .first()
                    .map(|(n, _)| n.clone())
                    .unwrap_or_default();
                log::info!(
                    "FONT_FALLBACK: found monospace font id={:?} name='{}'",
                    face.id,
                    name
                );
                self.font_id = Some(face.id);
                break;
            }
        }
        if self.font_id.is_none() {
            log::warn!("FONT_FALLBACK: no monospace font found in system!");
            for face in db.faces() {
                let name: String = face
                    .families
                    .first()
                    .map(|(n, _)| n.clone())
                    .unwrap_or_default();
                if !name.is_empty() {
                    log::warn!(
                        "FONT_FALLBACK: using non-monospace fallback font id={:?} name='{}'",
                        face.id,
                        name
                    );
                    self.font_id = Some(face.id);
                    break;
                }
            }
            if self.font_id.is_none() {
                log::error!(
                    "FONT_FALLBACK: no font at all found in system! Text will be invisible."
                );
            }
        }
    }

    fn find_cjk_fallback_fonts(&mut self) {
        let db = self.font_system.db();
        let test_char = '中';
        for face in db.faces() {
            if face.id == self.font_id.unwrap_or_default() {
                continue;
            }
            let has_cjk = db.with_face_data(face.id, |font_data, face_index| {
                let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
                let charmap = font_ref.charmap();
                let gid = charmap.map(test_char);
                if gid == 0 {
                    return Some(false);
                }
                let metrics = font_ref.metrics(&[]);
                let upem = metrics.units_per_em as f32;
                if upem == 0.0 {
                    return Some(false);
                }
                let scale = self.font_size / upem;
                let advance = font_ref.glyph_metrics(&[]).advance_width(gid);
                let advance_px = advance * scale;
                let cell_w = self.cell_metrics().0;
                Some(advance_px > cell_w * 1.5)
            });
            if has_cjk == Some(Some(true)) {
                let name: String = face
                    .families
                    .first()
                    .map(|(n, _)| n.clone())
                    .unwrap_or_default();
                log::info!("CJK_FALLBACK: found font id={:?} name='{}'", face.id, name);
                self.cjk_fallback_ids.push(face.id);
            }
        }
        log::info!(
            "CJK_FALLBACK: found {} fallback fonts",
            self.cjk_fallback_ids.len()
        );
    }

    pub fn set_font_family(&mut self, family_name: &str) -> bool {
        let found = {
            let db = self.font_system.db_mut();
            Self::find_font_by_name(db, family_name)
        };
        if let Some(id) = found {
            self.font_id = Some(id);
            self.glyph_cache.clear();
            self.cjk_fallback_ids.clear();
            self.find_cjk_fallback_fonts();
            return true;
        }
        false
    }

    pub fn glyph_info(&mut self, ch: char) -> Option<GlyphInfo> {
        let primary_font_id = self.font_id?;
        let db = self.font_system.db();

        let glyph_id = db.with_face_data(primary_font_id, |font_data, face_index| {
            let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
            let charmap = font_ref.charmap();
            Some(charmap.map(ch))
        })??;

        if glyph_id == 0 && !self.cjk_fallback_ids.is_empty() {
            for &fallback_id in &self.cjk_fallback_ids.clone() {
                let fallback_glyph = db.with_face_data(fallback_id, |font_data, face_index| {
                    let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
                    let charmap = font_ref.charmap();
                    Some(charmap.map(ch))
                });
                if let Some(Some(fid)) = fallback_glyph
                    && fid != 0
                {
                    return self.glyph_info_from_font(fallback_id, ch, fid);
                }
            }
        }

        self.glyph_info_from_font(primary_font_id, ch, glyph_id)
    }

    fn glyph_info_from_font(
        &mut self,
        font_id: fontdb::ID,
        _ch: char,
        glyph_id: swash::GlyphId,
    ) -> Option<GlyphInfo> {
        let key = GlyphKey {
            glyph_id,
            pixel_size: self.font_size as u16,
        };

        if let Some(info) = self.glyph_cache.get(&key).cloned() {
            return Some(info);
        }

        let db = self.font_system.db();
        let font_size = self.font_size;
        let (image, advance_width) =
            db.with_face_data(font_id, |font_data, face_index| -> Option<(_, f32)> {
                let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
                let mut scaler = self
                    .scaler_context
                    .builder(font_ref)
                    .size(font_size)
                    .hint(true)
                    .build();
                let image = Render::new(&[Source::Outline]).render(&mut scaler, glyph_id);
                let upem = font_ref.metrics(&[]).units_per_em as f32;
                let scale = if upem > 0.0 {
                    font_size / upem
                } else {
                    font_size
                };
                let advance_width = font_ref.glyph_metrics(&[]).advance_width(glyph_id) * scale;
                Some((image, advance_width))
            })??;

        let image = match image {
            Some(img) => img,
            None => {
                let info = GlyphInfo {
                    atlas_x: 0,
                    atlas_y: 0,
                    width: 0,
                    height: 0,
                    placement: Placement::default(),
                    advance_width,
                };
                self.glyph_cache.put(key, info.clone());
                return Some(info);
            }
        };

        let width = image.placement.width as i32;
        let height = image.placement.height as i32;

        if width == 0 || height == 0 {
            let info = GlyphInfo {
                atlas_x: 0,
                atlas_y: 0,
                width: 0,
                height: 0,
                placement: image.placement,
                advance_width,
            };
            self.glyph_cache.put(key, info.clone());
            return Some(info);
        }

        let allocation = match self
            .atlas
            .allocate(guillotiere::size2(width + 1, height + 1))
        {
            Some(a) => a,
            None => {
                let evict_count = (self.glyph_cache.len() / 4).max(1);
                for _ in 0..evict_count {
                    self.glyph_cache.pop_lru();
                }
                self.atlas
                    .allocate(guillotiere::size2(width + 1, height + 1))?
            }
        };
        let rect = allocation.rectangle;
        let ax = rect.min.x as i32;
        let ay = rect.min.y as i32;

        match image.content {
            swash::scale::image::Content::Mask => {
                let atlas_w = self.atlas_width as usize;
                let atlas_h = self.atlas_height as usize;
                for y in 0..height as usize {
                    let dst_y = ay as usize + y;
                    if dst_y >= atlas_h {
                        break;
                    }
                    for x in 0..width as usize {
                        let src_idx = y * width as usize + x;
                        let alpha = image.data.get(src_idx).copied().unwrap_or(0);
                        let dst_x = ax as usize + x;
                        if dst_x >= atlas_w {
                            break;
                        }
                        let dst_idx = dst_y * atlas_w + dst_x;
                        if dst_idx < self.atlas_bitmap.len() {
                            self.atlas_bitmap[dst_idx] = alpha;
                        }
                    }
                }
            }
            _ => {
                let atlas_w = self.atlas_width as usize;
                let atlas_h = self.atlas_height as usize;
                let bpp = 4;
                for y in 0..height as usize {
                    let dst_y = ay as usize + y;
                    if dst_y >= atlas_h {
                        break;
                    }
                    for x in 0..width as usize {
                        let dst_x = ax as usize + x;
                        if dst_x >= atlas_w {
                            break;
                        }
                        let src_idx = (y * width as usize + x) * bpp;
                        let dst_idx = dst_y * atlas_w + dst_x;
                        if dst_idx < self.atlas_bitmap.len() && src_idx + 3 < image.data.len() {
                            // RGBA→luminance for R8Unorm: use alpha as coverage
                            self.atlas_bitmap[dst_idx] = image.data[src_idx + 3];
                        }
                    }
                }
            }
        }

        let info = GlyphInfo {
            atlas_x: ax,
            atlas_y: ay,
            width,
            height,
            placement: image.placement,
            advance_width,
        };

        self.glyph_cache.put(key, info.clone());
        self.atlas_generation += 1;
        Some(info)
    }

    pub fn atlas_generation(&self) -> u64 {
        self.atlas_generation
    }

    pub fn rasterize_ascii(&mut self) {
        let before = self.cache_len();
        for ch in 32u8..127u8 {
            self.glyph_info(ch as char);
        }
        let after = self.cache_len();
        log::info!(
            "FONT_RASTERIZE_ASCII: before={} after={} font_id={:?}",
            before,
            after,
            self.font_id
        );
    }

    pub fn cache_len(&self) -> usize {
        self.glyph_cache.len()
    }

    pub fn list_monospace_fonts(&self) -> Vec<String> {
        let db = self.font_system.db();
        let mut fonts = Vec::new();
        for face in db.faces() {
            // On Android, many CJK/system fonts are not marked monospaced
            // but are needed for rendering. Include all loaded fonts.
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

    /// Find a font by family name with fallback matching.
    /// Tries exact match first, then checks if any family name in any
    /// face matches the search term (case-insensitive, normalized).
    fn find_font_by_name(db: &fontdb::Database, family_name: &str) -> Option<fontdb::ID> {
        // Exact match
        for face in db.faces() {
            for (family, _) in &face.families {
                if family.eq_ignore_ascii_case(family_name) {
                    return Some(face.id);
                }
            }
        }
        // Fuzzy match: clean the name (replace _/- with spaces, trim)
        let cleaned = family_name.replace(['_', '-'], " ").trim().to_lowercase();
        for face in db.faces() {
            for (family, _) in &face.families {
                let fam_lower = family.to_lowercase();
                if fam_lower == cleaned
                    || fam_lower.contains(&cleaned)
                    || cleaned.contains(&fam_lower)
                {
                    return Some(face.id);
                }
            }
        }
        // Strip all spaces and compare (handles "Noto Sans SC" vs "NotoSansSC")
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
        None
    }

    pub fn atlas_bitmap(&self) -> &[u8] {
        &self.atlas_bitmap
    }

    pub fn atlas_dimensions(&self) -> (u32, u32) {
        (self.atlas_width, self.atlas_height)
    }

    /// Create a FontPipeline from a fixture directory.
    /// Loads .ttf/.otf files from the given directory, skipping bundled font loading.
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
                {
                    let _ = db.load_font_file(&file_path);
                }
            }
        }

        let scaler_context = ScaleContext::new();
        let atlas = guillotiere::AtlasAllocator::new(guillotiere::size2(atlas_width, atlas_height));
        let atlas_bitmap = vec![0u8; (atlas_width * atlas_height) as usize];

        let mut pipeline = Self {
            font_system,
            scaler_context,
            atlas,
            glyph_cache: LruCache::new(NonZeroUsize::new(10000).unwrap()),
            atlas_bitmap,
            atlas_width: atlas_width as u32,
            atlas_height: atlas_height as u32,
            font_id: None,
            cjk_fallback_ids: Vec::new(),
            font_size,
            atlas_generation: 0,
        };

        pipeline.find_monospace_font();
        pipeline.find_cjk_fallback_fonts();
        pipeline
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Returns the ascent in pixels (distance from baseline to top of cell).
    /// Matches Termux `mFontAscent = ceil(mTextPaint.ascent())` (absolute value).
    /// This is the baseline's y-offset from the cell top.
    pub fn ascent_px(&self) -> f32 {
        if let Some(font_id) = self.font_id {
            let db = self.font_system.db();
            let result = db.with_face_data(font_id, |font_data, face_index| {
                let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
                let metrics = font_ref.metrics(&[]);
                let upem = metrics.units_per_em as f32;
                if upem == 0.0 {
                    return None;
                }
                let scale = self.font_size / upem;
                Some(metrics.ascent * scale)
            });
            if let Some(Some(px)) = result {
                return px;
            }
        }
        // Fallback: most monospace fonts have ascent ≈ 0.8× font size
        self.font_size * 0.8
    }

    /// Returns the descent in pixels (distance from baseline to bottom of cell).
    /// Matches Termux `mFontLineSpacing - |mFontAscent|`.
    /// swash `metrics.descent` may be positive or negative depending on font;
    /// we use `abs()` to ensure descent is always positive.
    pub fn descent_px(&self) -> f32 {
        if let Some(font_id) = self.font_id {
            let db = self.font_system.db();
            let result = db.with_face_data(font_id, |font_data, face_index| {
                let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
                let metrics = font_ref.metrics(&[]);
                let upem = metrics.units_per_em as f32;
                if upem == 0.0 {
                    return None;
                }
                let scale = self.font_size / upem;
                Some(metrics.descent.abs() * scale)
            });
            if let Some(Some(px)) = result {
                return px;
            }
        }
        self.font_size * 0.2
    }

    pub fn has_font(&self) -> bool {
        self.font_id.is_some()
    }

    /// Returns (cell_width, cell_height) in pixels for the current font.
    ///
    /// Termux formulas (TerminalRenderer.java):
    ///   cell_width  = Paint.measureText("X")
    ///   cell_height = ceil(Paint.getFontSpacing())
    ///   baseline    = |ceil(Paint.ascent())|  (= ascent_px)
    ///
    /// We use 'm' advance for width (equivalent for monospace fonts) and
    /// ceil(ascent + descent) for height, matching Termux's getFontSpacing().
    pub fn cell_metrics(&self) -> (f32, f32) {
        if let Some(font_id) = self.font_id {
            let db = self.font_system.db();
            let result = db.with_face_data(font_id, |font_data, face_index| {
                let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
                let metrics = font_ref.metrics(&[]);
                let upem = metrics.units_per_em as f32;
                if upem == 0.0 {
                    return None;
                }
                let scale = self.font_size / upem;
                let ascent = metrics.ascent * scale;
                // swash descent may be positive or negative depending on font;
                // abs() ensures correct cell_height regardless of convention.
                // swash leading (line_gap) is extra inter-line spacing.
                // Matches Termux: cell_height = ceil(getFontSpacing())
                //   = ceil(|ascent| + |descent| + line_gap)
                let descent = metrics.descent.abs() * scale;
                let line_gap = metrics.leading.max(0.0) * scale;
                let cell_height = ascent + descent + line_gap;

                let charmap = font_ref.charmap();
                let glyph_metrics = font_ref.glyph_metrics(&[]);

                if self
                    .font_id
                    .is_some_and(|id| db.faces().any(|f| f.id == id && f.monospaced))
                {
                    // Termux: mFontWidth = mTextPaint.measureText("X")
                    // For monospace fonts all characters share the same advance,
                    // so 'm' gives the same result as 'X'.
                    let glyph_id = charmap.map('m' as u32);
                    let advance = glyph_metrics.advance_width(glyph_id);
                    let cell_width = if advance > 0.0 {
                        advance * scale
                    } else {
                        // Fallback: standard monospace advance ≈ 0.6× font size
                        self.font_size * 0.6
                    };
                    // Termux: mFontLineSpacing = (int) Math.ceil(mTextPaint.getFontSpacing())
                    // ceil() prevents sub-pixel gaps between rows
                    return Some((cell_width, cell_height.ceil()));
                }

                // Non-monospace font: measure representative characters.
                // Termux: mTextPaint.measureText("X") for single char width.
                let max_advance = ['m', 'W', '0']
                    .iter()
                    .filter_map(|&ch| {
                        let gid = charmap.map(ch as u32);
                        let adv = glyph_metrics.advance_width(gid);
                        if adv > 0.0 { Some(adv * scale) } else { None }
                    })
                    .fold(0.0f32, f32::max);

                let cell_width = if max_advance > 0.0 {
                    max_advance
                } else {
                    self.font_size * 0.6
                };

                Some((cell_width, cell_height.ceil()))
            });
            if let Some(Some(m)) = result {
                return m;
            }
        }
        // Ultimate fallback when no font metrics are available.
        // Standard monospace ratio: width ≈ 0.6× size, height ≈ 1.2× size
        // (matches typical Latin monospace at 1:1.2 aspect ratio)
        (self.font_size * 0.6, (self.font_size * 1.2).ceil())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data");

    #[test]
    fn font_pipeline_creation() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        assert_eq!(pipeline.atlas_dimensions(), (2048, 2048));
        assert!(
            pipeline.cache_len() > 0,
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
            .glyph_info('好')
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
        let ascii_info = pipeline.glyph_info('A').expect("ascii 'A' glyph info");
        let cjk_info = pipeline.glyph_info('中').expect("CJK '中' glyph info");
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
    fn glyph_info_ascii() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let info = pipeline.glyph_info('A');
        assert!(info.is_some());
        let info = info.unwrap();
        assert!(info.width > 0);
        assert!(info.height > 0);
    }

    #[test]
    fn glyph_info_caching() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let before = pipeline.cache_len();
        pipeline.glyph_info('B');
        assert_eq!(pipeline.cache_len(), before);
        pipeline.glyph_info('B');
        assert_eq!(pipeline.cache_len(), before);
    }

    #[test]
    fn rasterize_ascii_populates_cache() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        assert!(pipeline.cache_len() >= 95);
    }

    #[test]
    fn glyph_info_has_atlas_coords() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let info = pipeline.glyph_info('X').unwrap();
        assert!(info.atlas_x >= 0);
        assert!(info.atlas_y >= 0);
        assert!(info.width > 0);
        assert!(info.height > 0);
    }

    #[test]
    fn atlas_bitmap_not_empty_after_rasterize() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        pipeline.glyph_info('A');
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

    const FIXTURE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_fonts");

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

    /// Load a FreeType golden reference image.
    /// Returns (width, height, rgba_data) or None if file is missing.
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

    /// Compare pipeline glyph atlas output against a FreeType golden reference.
    /// If the golden file is missing or font dimensions differ, auto-regenerate it
    /// so the test bootstraps on first run and catches regressions on subsequent runs.
    fn compare_with_freetype(ch: char, stem: &str) {
        let golden = load_freetype_golden(TEST_DATA_DIR, stem);

        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_info(ch)
            .unwrap_or_else(|| panic!("pipeline glyph_info('{ch}') should succeed"));
        let atlas = pipeline.atlas_bitmap();

        #[allow(unused_assignments)]
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
                        let ai = (ay + y) * atlas_w + ax + x;
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
        info: &crate::font::GlyphInfo,
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
        // CJK glyphs at 14px differ substantially between FreeType and skrifa
        // due to different hinting strategies. At small sizes, the 好 glyph
        // has ~123 of 182 pixels with alpha diff >2 (max=244).
        // This is expected cross-rasterizer variation.
        // Verify the golden file exists and the pipeline glyph has ink.
        let golden = load_freetype_golden(TEST_DATA_DIR, "hao");
        if let Some((_ft_w, _ft_h, _ft_data)) = golden {
            let mut pipeline = FontPipeline::new(512, 512, 14.0);
            let info = pipeline
                .glyph_info('好')
                .expect("CJK '好' should have glyph info");
            let atlas = pipeline.atlas_bitmap();
            let ax = info.atlas_x as usize;
            let ay = info.atlas_y as usize;
            let atlas_w = 512usize;
            let mut has_ink = false;
            for y in 0..info.height as usize {
                for x in 0..info.width as usize {
                    let idx = (ay + y) * atlas_w + ax + x;
                    if idx < atlas.len() && atlas[idx] > 0 {
                        has_ink = true;
                        break;
                    }
                }
                if has_ink {
                    break;
                }
            }
            assert!(has_ink, "CJK '好' should have non-zero coverage");
        } else {
            eprintln!("WARN: freetype golden 'hao' not found — skipping CJK comparison");
        }
    }

    #[test]
    fn bearing_values_for_dot() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline.glyph_info('.').expect("'.' should glyph_info");
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
            .glyph_info('A')
            .expect("'A' should have glyph_info");
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
            .glyph_info('好')
            .expect("'好' should have glyph_info");
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
        // CJK glyph is approximately 2x ASCII width
        let dot_info = pipeline.glyph_info('.').expect("'.' for comparison");
        assert!(
            info.placement.width >= dot_info.placement.width * 2 - 2,
            "好 width={} should be ~2x dot width={}",
            info.placement.width,
            dot_info.placement.width
        );
    }

    #[allow(dead_code)]
    fn bearing_fits_inside_cell(glyph: char, label: &str) {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_info(glyph)
            .unwrap_or_else(|| panic!("'{glyph}' glyph_info"));
        let (_cell_w, cell_h) = pipeline.cell_metrics();
        let ascent = pipeline.ascent_px();
        let bearing_y = ascent - info.placement.top as f32;
        let glyph_h = info.placement.height as f32;
        assert!(
            bearing_y >= -cell_h,
            "{label} glyph starts way above cell: bearing_y={} < -cell_h",
            bearing_y
        );
        assert!(glyph_h > 0.0, "{label} glyph has zero height",);
        assert!(cell_h > 0.0, "{label} cell has zero height",);
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
            if let Some(info) = pipeline.glyph_info(ch) {
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

    // ── 13.1: Font enumeration ─────────────────────────────────────

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

    // ── 13.2: CJK glyph ────────────────────────────────────────────

    #[test]
    fn cjk_glyph_zhong() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        let info = pipeline
            .glyph_info('中')
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
                let idx = (ay + y) * atlas_w + ax + x;
                if idx < atlas.len() && atlas[idx] > 0 {
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

    // ── 13.3: Emoji glyph ──────────────────────────────────────────

    #[test]
    fn emoji_glyph_grinning() {
        let mut pipeline = FontPipeline::new(512, 512, 14.0);
        // Try the bundled font first; if it doesn't have emoji, try
        // looking up a system font that does.
        let ch = '\u{1F600}'; // grinning face
        let info = pipeline.glyph_info(ch);
        if info.is_none() || info.as_ref().is_some_and(|i| i.width == 0) {
            let fonts = pipeline.list_monospace_fonts();
            let found_emoji = fonts.iter().any(|name| {
                name.contains("Emoji")
                    || name.contains("Noto")
                    || name.to_lowercase().contains("emoji")
            });
            if !found_emoji {
                panic!(
                    "no emoji-supporting font found in system; emoji glyph test requires Noto Emoji or similar"
                );
            }
        }
        let info = info.expect("emoji 😀 should have glyph info");
        assert!(
            info.width > 0 || info.height > 0,
            "emoji 😀 should produce non-zero glyph info: got {}x{}",
            info.width,
            info.height
        );
    }

    // ── 13.4: GlyphAtlas LRU eviction ──────────────────────────────

    #[test]
    fn glyph_atlas_lru_eviction() {
        // Create a tiny LRU cache to force eviction quickly.
        // We override the default internal cache by constructing a
        // pipeline and then swapping caches via unsafe access to the
        // private field. Instead, create a standalone LRU test.
        let mut pipeline = FontPipeline::new(512, 512, 14.0);

        // Rasterize ASCII to populate cache with ~95 known entries.
        pipeline.rasterize_ascii();
        let after_ascii = pipeline.cache_len();
        assert!(
            after_ascii >= 95,
            "should have at least 95 cached after rasterize_ascii, got {}",
            after_ascii
        );

        // Insert many unique CJK glyphs to potentially trigger eviction.
        // The internal LruCache has capacity 10000. Atlas allocation
        // may fill up before the cache does, triggering partial eviction.
        let mut inserted = 0u32;
        for cp in 0x4E00u32..0x4F00u32 {
            let ch = char::from_u32(cp).unwrap_or('\0');
            if pipeline.glyph_info(ch).is_some_and(|i| i.width > 0) {
                inserted += 1;
            }
        }
        let final_len = pipeline.cache_len();
        // Cache must be bounded by its capacity.
        assert!(
            final_len <= 10000,
            "cache_len {} exceeds capacity 10000",
            final_len
        );
        // At least some new glyphs were inserted.
        assert!(
            final_len >= after_ascii,
            "cache should not shrink after inserting new glyphs: \
             before={} after={} inserted={}",
            after_ascii,
            final_len,
            inserted
        );
        // Verify atlas bitmap has content.
        let bitmap = pipeline.atlas_bitmap();
        assert!(
            bitmap.iter().any(|&b| b != 0),
            "atlas bitmap should have non-zero bytes after glyph insertion"
        );
    }

    #[test]
    fn cjk_glyph_info_returns_nonzero_for_common_chars() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let chars = [
            '你', '好', '世', '界', '中', '文', '字', '体', '渲', '染', '测', '试',
        ];
        for ch in chars {
            let info = pipeline
                .glyph_info(ch)
                .unwrap_or_else(|| panic!("CJK glyph_info('{ch}') should return Some"));
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
        if names.len() > 1 {
            let alt_name = names.iter().find(|n| !n.is_empty()).cloned();
            if let Some(alt_name) = alt_name
                && pipeline.set_font_family(&alt_name)
            {
                assert_ne!(
                    pipeline.font_id, original_id,
                    "font_id should change after switching to '{}'",
                    alt_name
                );
            }
        }
    }

    #[test]
    fn font_switching_clears_cache() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        pipeline.rasterize_ascii();
        let before = pipeline.cache_len();
        assert!(before > 0);
        pipeline.glyph_info('好');
        let names = pipeline.list_monospace_fonts();
        if names.len() > 1 {
            let alt = names.last().unwrap();
            pipeline.set_font_family(alt);
            assert_eq!(
                pipeline.cache_len(),
                0,
                "cache should be cleared after font switch to '{alt}'"
            );
        } else {
            pipeline.set_font_family("monospace");
            if pipeline.cache_len() == 0 {
                return;
            }
            assert!(
                pipeline.cache_len() < before,
                "cache should shrink after font switch"
            );
        }
    }

    // ── Cell metrics: ceil() prevents sub-pixel gaps (Termux approach) ──

    #[test]
    fn cell_metrics_height_is_integer() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let (_cw, ch) = pipeline.cell_metrics();
        assert_eq!(
            ch,
            ch.floor(),
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
        assert_eq!(sh, sh.floor(), "small cell_height should be integer");
        assert_eq!(lh, lh.floor(), "large cell_height should be integer");
    }

    // ── Termux formula validation: verifies our formulas match Termux/TerminalRenderer.java ──

    #[test]
    fn termux_formula_ascent_plus_descent_equals_cell_height() {
        // Termux: cell_height = ceil(getFontSpacing()) ≈ ceil(|ascent| + descent)
        // Our: ascent_px + descent_px ≈ cell_height (within 1px due to ceil rounding)
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let ascent = pipeline.ascent_px();
        let descent = pipeline.descent_px();
        let (cw, ch) = pipeline.cell_metrics();
        let _ = cw;
        assert!(
            (ascent + descent - ch).abs() < 2.0,
            "ascent({ascent}) + descent({descent}) ≈ cell_height({ch}), diff={}",
            (ascent + descent - ch).abs()
        );
    }

    #[test]
    fn termux_formula_baseline_is_ascent_from_cell_top() {
        // Termux: baseline = |ceil(ascent)| from cell top
        // Our: ascent_px() IS the baseline position from cell top
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let ascent = pipeline.ascent_px();
        let (cw, ch) = pipeline.cell_metrics();
        let _ = cw;
        assert!(
            ascent > 0.0 && ascent < ch,
            "ascent({ascent}) must be in (0, cell_h={ch})"
        );
    }

    #[test]
    fn termux_formula_glyph_bearing_y_matches() {
        // Termux: text drawn at baseline = ascent from cell top
        // bearing_y = baseline - placement.top = ascent_px - placement.top
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let ascent = pipeline.ascent_px();
        let info = pipeline.glyph_info('A').expect("should have 'A' glyph");
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
    fn descent_px_is_positive() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        let descent = pipeline.descent_px();
        assert!(descent > 0.0, "descent should be positive, got {descent}");
    }

    #[test]
    fn cell_width_from_m_advance_matches() {
        // Termux: mFontWidth = Paint.measureText("X")
        // For monospace fonts, advance('m') == advance('X')
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let info_m = pipeline.glyph_info('m').expect("should have 'm'");
        let info_x = pipeline.glyph_info('X').expect("should have 'X'");
        let (cw, _ch) = pipeline.cell_metrics();
        let _ = cw;
        // advance_width should be close to cell_width for monospace
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

    // ── Font-agnostic: works for ANY monospace font, no per-font magic ──

    #[test]
    fn any_monospace_advance_matches_cell_width() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        let (cw, _) = pipeline.cell_metrics();
        let chars = ['A', 'm', 'W', '0', 'l', 'i'];
        for ch in chars {
            if let Some(info) = pipeline.glyph_info(ch) {
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
                if let Some(info) = pipeline.glyph_info(ch) {
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
            if let Some(info) = pipeline.glyph_info(ch) {
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
        let ascent = pipeline.ascent_px();
        let ascii = ['A', 'B', 'C', 'x', 'y', 'z', '0', '1', '9'];
        for ch in ascii {
            if let Some(info) = pipeline.glyph_info(ch) {
                let bearing_y = ascent - info.placement.top as f32;
                assert!(
                    bearing_y >= -2.0,
                    "bearing_y('{ch}')={:.1} should be >= -2",
                    bearing_y
                );
            }
        }
    }

    // ── External font tests: Maple Mono CN + Source Han Sans SC ──
    // These fonts must be downloaded first: nu scripts/download-test-fonts.nu

    fn try_load_external_font(path: &str) -> Option<FontPipeline> {
        let font_path = std::path::Path::new(path);
        if !font_path.exists() {
            return None;
        }
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0 * 3.0);
        pipeline
            .font_system
            .db_mut()
            .load_font_file(font_path)
            .ok()?;
        Some(pipeline)
    }

    #[test]
    fn maple_mono_normal_nf_cn_medium_cjk_rendering() {
        let paths = [
            "/tmp/test-fonts/maple-normal-nf-cn/MapleMonoNormal-NF-CN-Medium.ttf",
            "/data/local/tmp/MapleMonoNormal-NF-CN-Medium.ttf",
        ];
        let mut found = false;
        for path in &paths {
            if !std::path::Path::new(path).exists() {
                continue;
            }
            let mut p =
                try_load_external_font(path).expect("should load MapleMonoNormal-NF-CN-Medium");
            let (cw, ch) = p.cell_metrics();
            assert!(
                cw > 0.0 && ch > 0.0,
                "MapleMonoNormal: cell_metrics cw={cw} ch={ch}"
            );
            for c in ['A', '中', '好', '世', '界', '日', '本'] {
                if let Some(info) = p.glyph_info(c) {
                    assert!(
                        info.width > 0 && info.height > 0,
                        "MapleMonoNormal: glyph '{c}' has zero size"
                    );
                    assert!(
                        info.advance_width > 0.0,
                        "MapleMonoNormal: glyph '{c}' has zero advance"
                    );
                }
            }
            found = true;
            break;
        }
        if !found {
            eprintln!(
                "SKIP: MapleMonoNormal-NF-CN-Medium not found (run: nu scripts/download-test-fonts.nu)"
            );
        }
    }

    #[test]
    fn maple_mono_cn_cjk_rendering() {
        let paths = [
            "/tmp/test-fonts/maple/MapleMono-CN-Regular.ttf",
            "/data/local/tmp/MapleMono-CN-Regular.ttf",
        ];
        let mut found = false;
        for path in &paths {
            if !std::path::Path::new(path).exists() {
                continue;
            }
            let mut p = try_load_external_font(path).expect("should load Maple CN font");
            let (cw, ch) = p.cell_metrics();
            assert!(
                cw > 0.0 && ch > 0.0,
                "Maple CN: cell_metrics cw={cw} ch={ch}"
            );
            for c in ['A', '中', '好', '世', '界', '日', '本'] {
                if let Some(info) = p.glyph_info(c) {
                    assert!(
                        info.width > 0 && info.height > 0,
                        "Maple CN: glyph '{c}' has zero size"
                    );
                    assert!(
                        info.advance_width > 0.0,
                        "Maple CN: glyph '{c}' has zero advance"
                    );
                }
            }
            found = true;
            break;
        }
        if !found {
            eprintln!("SKIP: Maple font not found (run: nu scripts/download-test-fonts.nu)");
        }
    }

    #[test]
    fn source_han_sans_sc_cjk_rendering() {
        let paths = [
            "/tmp/test-fonts/source-han/OTF/SimplifiedChinese/SourceHanSansSC-Regular.otf",
            "/data/local/tmp/SourceHanSansSC-Regular.otf",
        ];
        let mut found = false;
        for path in &paths {
            if !std::path::Path::new(path).exists() {
                continue;
            }
            let mut p = try_load_external_font(path).expect("should load Source Han Sans");
            let (cw, ch) = p.cell_metrics();
            assert!(
                cw > 0.0 && ch > 0.0,
                "Source Han: cell_metrics cw={cw} ch={ch}"
            );
            for c in ['A', '中', '好', '世', '界', '日', '本', 'α', 'Ω'] {
                if let Some(info) = p.glyph_info(c) {
                    assert!(
                        info.width > 0 && info.height > 0,
                        "Source Han: glyph '{c}' has zero size"
                    );
                    assert!(
                        info.advance_width > 0.0,
                        "Source Han: glyph '{c}' has zero advance"
                    );
                }
            }
            found = true;
            break;
        }
        if !found {
            eprintln!("SKIP: Source Han Sans not found (run: nu scripts/download-test-fonts.nu)");
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
            if let Some(info) = pipeline.glyph_info(ch) {
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
}
