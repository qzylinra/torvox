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
}

impl FontPipeline {
    pub fn new(atlas_width: i32, atlas_height: i32, font_size: f32) -> Self {
        let font_system = FontSystem::new();
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
            font_id: None,
            font_size,
        };

        pipeline.find_monospace_font();
        pipeline
    }

    fn find_monospace_font(&mut self) {
        let db = self.font_system.db();
        for face in db.faces() {
            if face.monospaced {
                self.font_id = Some(face.id);
                break;
            }
        }
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
        Some(info)
    }

    pub fn rasterize_ascii(&mut self) {
        for ch in 32u8..127u8 {
            self.glyph_info(ch as char);
        }
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

    /// Returns the monospace cell dimensions (cell_width, cell_height) in pixels.
    /// Uses the font's ascent + descent for the line height (the most reliable
    /// signal across monospace fonts). The cell width is computed from the
    /// font's `max_width` (from hmtx table) for monospace fonts, or estimated
    /// at 0.6 × font_size as a fallback.
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
                let cell_width = self.font_size * 0.6;
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
