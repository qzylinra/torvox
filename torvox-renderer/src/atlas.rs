pub struct GlyphAtlas {
    atlas: etagere::AtlasAllocator,
}

impl GlyphAtlas {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            atlas: etagere::AtlasAllocator::new(etagere::size2(width, height)),
        }
    }

    pub fn allocate(&mut self, width: i32, height: i32) -> Option<etagere::Allocation> {
        self.atlas.allocate(etagere::size2(width, height))
    }
}
