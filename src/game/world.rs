use crate::{
    util::{Point2},
    io::tex::{Assets, },
};
use ggez::{
    Context, GameResult,
    graphics,
};

mod material;
pub use material::*;

#[derive(Debug)]
/// All the objects in the current world
pub struct World {
    pub grid: Grid,
}

impl World {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            grid: Grid::new(width, height),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Material {
    Apples,
    Grains,
    Lumber,
    Ore,
    Sheeps,
}

#[derive(Debug, Clone)]
pub struct Grid {
    width: u16,
    mats: Vec<Material>,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        Grid {
            width,
            mats: vec![Material::Apples; (width*height) as usize],
        }
    }
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }
    pub fn height(&self) -> u16 {
        self.mats.len() as u16 / self.width
    }
    pub fn widen(&mut self) {
        let width = self.width as usize;
        let height = self.height() as usize;
        self.mats.reserve_exact(height);
        for i in (1..=height).rev().map(|i| i * width) {
            self.mats.insert(i, Material::Apples);
        }
        self.width += 1;
    }
    pub fn thin(&mut self) {
        if self.width <= 1 {
            return
        }
        let width = self.width;
        for i in (1..=self.height()).rev().map(|i| i * width - 1) {
            self.mats.remove(i as usize);
        }
        self.width -= 1;
    }
    pub fn heighten(&mut self) {
        let new_len = self.mats.len() + self.width as usize;
        self.mats.reserve_exact(self.width as usize);
        self.mats.resize(new_len, Material::Apples);
    }
    pub fn shorten(&mut self) {
        let new_len = self.mats.len() - self.width as usize;
        if new_len == 0 {
            return
        }
        self.mats.truncate(new_len);
    }
    #[inline]
    pub fn snap(c: Point2) -> (u16, u16) {
        Self::snap_coords(c.x, c.y)
    }
    #[inline]
    fn idx(&self, x: u16, y: u16) -> usize {
        x.saturating_add(y.saturating_mul(self.width)) as usize
    }
    pub fn snap_coords(x: f32, y: f32) -> (u16, u16) {
        fn db32omin(n: f32) -> u16 {
            if n < 0. {
                std::u16::MAX
            } else {
                (n / 32.) as u16
            }
        }

        (db32omin(x), db32omin(y))
    }
    pub fn get(&self, x: u16, y: u16) -> Option<Material> {
        if x < self.width {
            self.mats.get(self.idx(x, y)).copied()
        } else {
            None
        }
    }
    pub fn insert(&mut self, x: u16, y: u16, mat: Material) {
        if x < self.width {
            let i = self.idx(x, y);
            if let Some(m) = self.mats.get_mut(i) {
                *m = mat;
            }
        }
    }
    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        for (i, &mat) in self.mats.iter().enumerate() {
            let x = f32::from(i as u16 % self.width) * 32.;
            let y = f32::from(i as u16 / self.width) * 32.;

            let mat = &format!("{:?}", mat).to_lowercase();

            let img = get_img(ctx, assets, mat);
            graphics::draw(ctx, &*img, (Point2::new(x, y),))?;
        }
        Ok(())
    }
}
