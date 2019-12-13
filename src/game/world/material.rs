use crate::{
    io::tex::Assets,
    util::{Point2, Vector2},
};
use ggez::{graphics::{self, Image}, Context, GameResult};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;
use std::fs::File;
use std::io::Read;
use std::cell::Ref;

#[derive(Debug)]
pub struct Mat {
    spr: Box<str>,
}

lazy_static! {
    static ref MATS: RwLock<HashMap<String, Mat>> = {
        RwLock::new(HashMap::with_capacity(10))
    };
}

fn ensure(mat: &str) {
    if !MATS.read().unwrap().contains_key(mat) {
        let mat_data = Mat { spr: format!("materials/{}", mat).into_boxed_str()};

        MATS.write().unwrap().insert(mat.to_owned(), mat_data);
    }
}

#[derive(Debug, Default)]
struct MaterialProperties {
    solid: bool,
}

#[inline]
pub fn get_img<'a>(ctx: &mut Context, assets: &'a Assets, mat: &str) -> Ref<'a, Image> {
    ensure(mat);

    assets.get_img(ctx, &MATS.read().unwrap()[mat].spr)
}

const PALETTE: &[&str] = &["apples", "grains", "lumber", "ore", "sheeps"];
