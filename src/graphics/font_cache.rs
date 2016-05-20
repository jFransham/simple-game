use std::collections::HashMap;
use sdl2_ttf::{Sdl2TtfContext, Font};

pub struct FontCache<'a> {
    context: &'a Sdl2TtfContext,
    cache: HashMap<(&'static str, u16), Font>
}

impl<'a> FontCache<'a> {
    pub fn new(context: &'a Sdl2TtfContext) -> Self {
        FontCache {
            context: context,
            cache: HashMap::new(),
        }
    }

    pub fn load(&mut self, path: &'static str, size: u16) -> Option<&Font> {
        self.with_loaded(path, size).ok().and_then(|s| s.get(path, size))
    }

    pub fn with_loaded(
        &mut self, path: &'static str, size: u16
    ) -> Result<&mut Self, String> {
        if self.cache.contains_key(&(path, size)) { return Ok(self); }

        match self.context.load_font(path.as_ref(), size) {
            Ok(font) => {
                self.cache.insert((path, size), font);

                Ok(self)
            },
            Err(e) => Err(e),
        }
    }

    pub fn get(&self, path: &'static str, size: u16) -> Option<&Font> {
        self.cache.get(&(path, size))
    }
}
