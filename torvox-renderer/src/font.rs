use std::num::NonZeroUsize;

use cosmic_text::FontSystem;
use lru::LruCache;
use swash::scale::{Render, ScaleContext, Source};
use swash::zeno::Placement;
use thiserror::Error;

const FONT_DATA: &[u8] = include_bytes!("../fonts/JetBrainsMonoNerdFont-Regular.ttf");

#[derive(Clone)]
struct FontBytes(&'static [u8]);
impl AsRef<[u8]> for FontBytes {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

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
    font_size: f32,
    atlas_generation: u64,
}

impl FontPipeline {
    pub fn new(atlas_width: i32, atlas_height: i32, font_size: f32) -> Self {
        let mut font_system = FontSystem::new();
        let db = font_system.db_mut();
        let data: std::sync::Arc<dyn AsRef<[u8]> + std::marker::Send + std::marker::Sync> =
            std::sync::Arc::new(FontBytes(FONT_DATA));
        let ids = db.load_font_source(fontdb::Source::Binary(data));
        let bundled_font_id = ids.first().copied();
        log::info!(
            "FONT_LOAD: bundled font IDs={:?} (first={:?}), has_font={}",
            ids,
            bundled_font_id,
            bundled_font_id.is_some(),
        );
        let _ = db;
        let scaler_context = ScaleContext::new();
        let atlas = guillotiere::AtlasAllocator::new(guillotiere::size2(atlas_width, atlas_height));
        let atlas_bitmap = vec![0u8; (atlas_width * atlas_height * 4) as usize];

        let mut pipeline = Self {
            font_system,
            scaler_context,
            atlas,
            glyph_cache: LruCache::new(NonZeroUsize::new(10000).unwrap()),
            atlas_bitmap,
            atlas_width: atlas_width as u32,
            atlas_height: atlas_height as u32,
            font_id: bundled_font_id,
            font_size,
            atlas_generation: 0,
        };

        if pipeline.font_id.is_none() {
            pipeline.find_monospace_font();
        }
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
        }
    }

    pub fn set_font_family(&mut self, family_name: &str) -> bool {
        let db = self.font_system.db_mut();
        for face in db.faces() {
            for (family, _) in &face.families {
                if family.eq_ignore_ascii_case(family_name) {
                    self.font_id = Some(face.id);
                    self.glyph_cache.clear();
                    return true;
                }
            }
        }
        false
    }

    pub fn glyph_info(&mut self, ch: char) -> Option<GlyphInfo> {
        let font_id = self.font_id?;
        let db = self.font_system.db();

        let glyph_id = db.with_face_data(font_id, |font_data, face_index| {
            let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
            let charmap = font_ref.charmap();
            Some(charmap.map(ch))
        })??;

        let key = GlyphKey {
            glyph_id,
            pixel_size: self.font_size as u16,
        };

        if let Some(info) = self.glyph_cache.get(&key).cloned() {
            return Some(info);
        }

        let image = db.with_face_data(font_id, |font_data, face_index| {
            let font_ref = swash::FontRef::from_index(font_data, face_index as usize)?;
            let mut scaler = self
                .scaler_context
                .builder(font_ref)
                .size(self.font_size)
                .hint(true)
                .build();
            Render::new(&[Source::Outline]).render(&mut scaler, glyph_id)
        })?;

        let image = match image {
            Some(img) => img,
            None => {
                let info = GlyphInfo {
                    atlas_x: 0,
                    atlas_y: 0,
                    width: 0,
                    height: 0,
                    placement: Placement::default(),
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
                        let dst_idx = (dst_y * atlas_w + dst_x) * 4;
                        if dst_idx + 3 < self.atlas_bitmap.len() {
                            self.atlas_bitmap[dst_idx] = 255;
                            self.atlas_bitmap[dst_idx + 1] = 255;
                            self.atlas_bitmap[dst_idx + 2] = 255;
                            self.atlas_bitmap[dst_idx + 3] = alpha;
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
                        let dst_idx = (dst_y * atlas_w + dst_x) * 4;
                        if dst_idx + 3 < self.atlas_bitmap.len() && src_idx + 3 < image.data.len() {
                            self.atlas_bitmap[dst_idx] = image.data[src_idx];
                            self.atlas_bitmap[dst_idx + 1] = image.data[src_idx + 1];
                            self.atlas_bitmap[dst_idx + 2] = image.data[src_idx + 2];
                            self.atlas_bitmap[dst_idx + 3] = image.data[src_idx + 3];
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
            if face.monospaced {
                for (family, _) in &face.families {
                    let name = family.to_string();
                    if !fonts.contains(&name) {
                        fonts.push(name);
                    }
                }
            }
        }
        fonts.sort();
        fonts
    }

    pub fn atlas_bitmap(&self) -> &[u8] {
        &self.atlas_bitmap
    }

    pub fn atlas_dimensions(&self) -> (u32, u32) {
        (self.atlas_width, self.atlas_height)
    }

    pub fn has_font(&self) -> bool {
        self.font_id.is_some()
    }

    /// 返回等宽单元格尺寸（cell_width, cell_height），单位为像素。
    /// 使用字体的 ascent + descent 作为行高（跨等宽字体最可靠的指标）。
    /// 单元格宽度从字体的 `max_width`（hmtx 表）计算，或以 0.6 × font_size 作为回退。
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
                let descent = -metrics.descent * scale;
                let cell_height = if ascent + descent > 0.0 {
                    ascent + descent
                } else {
                    self.font_size * 1.2
                };
                let charmap = font_ref.charmap();
                let glyph_id = charmap.map('W' as u32);
                let advance = font_ref.glyph_metrics(&[]).advance_width(glyph_id);
                let cell_width = if advance > 0.0 {
                    advance * scale
                } else {
                    self.font_size * 0.6
                };
                Some((cell_width, cell_height))
            });
            if let Some(Some(m)) = result {
                return m;
            }
        }
        (self.font_size * 0.6, self.font_size * 1.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_pipeline_creation() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        assert_eq!(pipeline.atlas_dimensions(), (2048, 2048));
        assert_eq!(pipeline.cache_len(), 0);
    }

    #[test]
    fn font_pipeline_has_font() {
        let pipeline = FontPipeline::new(2048, 2048, 14.0);
        assert!(pipeline.has_font());
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
        pipeline.glyph_info('B');
        assert_eq!(pipeline.cache_len(), 1);
        pipeline.glyph_info('B');
        assert_eq!(pipeline.cache_len(), 1);
    }

    #[test]
    fn rasterize_ascii_populates_cache() {
        let mut pipeline = FontPipeline::new(2048, 2048, 14.0);
        pipeline.rasterize_ascii();
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
}
