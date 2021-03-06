use ::gameobjects::{Clip, Dest};
use ::time::TimeExtensions;

use std::rc::Rc;
use std::ops::Index;
use std::hash::Hash;
use std::path::Path;
use std::collections::HashMap;
use sdl2::render::{TextureQuery, Texture, Renderer};
use sdl2::pixels::Color;
use sdl2_image::LoadTexture;
use itertools::*;

pub trait GetSize {
    fn size(&self) -> [u32; 2];
}

pub trait CopyRenderable<T> {
    fn copy_renderable(&mut self, sprite: &T, dest: Dest);
}

impl<'a> CopyRenderable<Sprite<Texture>> for Renderer<'a> {
    fn copy_renderable(&mut self, sprite: &Sprite<Texture>, dest: Dest) {
        self.copy(
            &sprite.texture,
            Some(sprite.mask.into()),
            Some(dest.into())
        );
    }
}

impl<'a> CopyRenderable<VisibleRect> for Renderer<'a> {
    fn copy_renderable(
        &mut self,
        &VisibleRect(color): &VisibleRect,
        dest: Dest
    ) {
        self.set_draw_color(color);

        self.fill_rect(dest.into()).unwrap();
    }
}

impl<'a> CopyRenderable<VisibleComponent<Texture>> for Renderer<'a> {
    fn copy_renderable(
        &mut self,
        comp: &VisibleComponent<Texture>,
        dest: Dest
    ) {
        match *comp {
            VisibleComponent::Sprite(ref spr) =>
                self.copy_renderable(spr, dest),
            VisibleComponent::Rectangle(ref rect) =>
                self.copy_renderable(rect, dest),
        }
    }
}

pub trait Renderable<T> {
    fn render(&self, renderer: &mut T, dest: Dest);
}

impl<'a, T> Renderable<Renderer<'a>> for T where Renderer<'a>: CopyRenderable<T> {
    fn render(&self, renderer: &mut Renderer<'a>, dest: Dest) {
        renderer.copy_renderable(self, dest);
    }
}

impl GetSize for Texture {
    fn size(&self) -> [u32; 2] {
        let TextureQuery { width: w, height: h, .. } = self.query();

        [w, h]
    }
}

impl GetSize for Clip {
    fn size(&self) -> [u32; 2] {
        [self.width, self.height]
    }
}

pub fn build_spritesheet<T: GetSize, RcT: Into<Rc<T>>>(
    texture: RcT,
    sprite_width: u32,
    sprite_height: u32
) -> Vec<Sprite<T>> {
    let rctex = texture.into();

    let [w, h] = rctex.size();

    (0..h).step_by(sprite_height)
        .cartesian_product((0..w).step_by(sprite_width))
        .map(|(y, x)|
            Sprite::new_with_mask(
                rctex.clone(),
                Clip {
                    x: x,
                    y: y,
                    width: sprite_width,
                    height: sprite_height,
                }
            )
        ).collect()
}

#[derive(Debug)]
pub struct Sprite<T: GetSize> {
    pub texture: Rc<T>,
    pub mask: Clip,
}

pub struct VisibleRect(pub Color);

pub enum VisibleComponent<T: GetSize> {
    Sprite(Sprite<T>),
    Rectangle(VisibleRect),
}

impl<T: GetSize> From<Sprite<T>> for VisibleComponent<T> {
    fn from(sprite: Sprite<T>) -> Self {
        VisibleComponent::Sprite(sprite)
    }
}

impl<T: GetSize> From<VisibleRect> for VisibleComponent<T> {
    fn from(rect: VisibleRect) -> Self {
        VisibleComponent::Rectangle(rect)
    }
}

impl<T: GetSize> Clone for Sprite<T> {
    fn clone(&self) -> Self {
        Sprite {
            texture: self.texture.clone(),
            mask: self.mask,
        }
    }
}

impl<T: GetSize> Sprite<T> {
    pub fn with_mask(&self, mask: Clip) -> Self {
        Sprite {
            mask: mask,
            .. self.clone()
        }
    }

    pub fn new_with_mask<RcT: Into<Rc<T>>>(tex: RcT, mask: Clip) -> Self {
        Sprite {
            texture: tex.into(),
            mask: mask,
        }
    }

    pub fn new<RcT: Into<Rc<T>>>(tex: RcT) -> Self {
        let rc_tex = tex.into();

        let [w, h] = rc_tex.size();

        Sprite {
            texture: rc_tex,
            mask: Clip {
                width: w,
                height: h,
                .. Default::default()
            },
        }
    }
}

pub trait LoadSprite<T: GetSize> {
    fn load_sprite<P: AsRef<Path>>(&self, path: P) -> Result<Sprite<T>, String>;
}

impl<L: LoadTexture> LoadSprite<Texture> for L {
    fn load_sprite<P: AsRef<Path>>(
        &self, path: P
    ) -> Result<Sprite<Texture>, String> {
        self.load_texture(path.as_ref()).map(Sprite::new)
    }
}

pub struct AnimationSet<Idx: Eq + Hash, AIdx>(
    pub HashMap<Idx, Animation<AIdx>>
);

impl<Idx: Eq + Hash, AIdx> AnimationSet<Idx, AIdx> {
    pub fn from_tuples<
        A: Into<Animation<AIdx>>,
        I: IntoIterator<Item=(Idx, A)>
    >(iter: I) -> Self {
        let mut tree = HashMap::new();

        for (index, item) in iter {
            tree.insert(index, item.into());
        }

        AnimationSet(tree)
    }
}

pub struct Animation<AIdx> {
    pub fps: f64,
    pub frames: Vec<AIdx>,
}

impl<Idx, I: IntoIterator<Item=Idx>> From<(f64, I)> for Animation<Idx> {
    fn from((fps, iter): (f64, I)) -> Self {
        Animation {
            fps: fps,
            frames: iter.into_iter().collect(),
        }
    }
}

pub type AnimatedSprite<
    Time,
    T,
> = AnimatedSpriteSheet<(), usize, Time, T, Vec<Sprite<T>>>;

impl<
    Time: TimeExtensions + Clone,
    T: GetSize,
> AnimatedSprite<Time, T> {
    pub fn from_spritesheet(
        now: Time,
        fps: f64,
        spritesheet: Vec<Sprite<T>>
    ) -> Self {
        let mut hm = HashMap::new();

        hm.insert(
            (),
            Animation {
                fps: fps,
                frames: (0..spritesheet.len()).collect(),
            }
        );

        Self::new(
            spritesheet,
            AnimationSet(hm),
            (),
            now
        )
    }
}

pub struct AnimatedSpriteSheet<
    ASIdx: Eq + Hash,
    AIdx: Clone,
    Time: TimeExtensions + Clone,
    T: GetSize,
    C: Index<AIdx, Output=Sprite<T>>
> {
    sprites: C,
    animations: AnimationSet<ASIdx, AIdx>,
    current: (Time, ASIdx),
}

impl<
    ASIdx: Eq + Hash,
    AIdx: Clone,
    Time: TimeExtensions + Clone,
    T: GetSize,
    C: Index<AIdx, Output=Sprite<T>>
> AnimatedSpriteSheet<ASIdx, AIdx, Time, T, C> {
    pub fn new(
        sprites: C,
        animations: AnimationSet<ASIdx, AIdx>,
        initial: ASIdx,
        now: Time
    ) -> Self {
        AnimatedSpriteSheet {
            sprites: sprites,
            animations: animations,
            current: (now, initial),
        }
    }

    pub fn set_animation(&mut self, new: ASIdx, now: Time) {
        self.current = (now, new);
    }

    pub fn completed(&self, now: Time) -> bool {
        let anim = &self.animations.0[&self.current.1];
        let start = self.current.0.clone().exact_seconds();

        let len = anim.frames.len();
        let fps = anim.fps;

        now.exact_seconds() - start > len as f64 / fps
    }

    pub fn frame(&self, now: Time) -> &Sprite<T> {
        &self.sprites[self.frame_index(now)]
    }

    fn frame_index(&self, now: Time) -> AIdx {
        let anim = &self.animations.0[&self.current.1];

        let tick_diff =
            now.exact_seconds() - self.current.0.clone().exact_seconds();

        let frame_num = tick_diff * anim.fps;

        anim.frames[frame_num as usize % anim.frames.len()].clone()
    }
}
