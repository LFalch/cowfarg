// #![windows_subsystem = "windows"]
//! Shooter game
#![warn(clippy::all)]

#[macro_use]
extern crate log;

use std::env::args;

use ggez::{
    ContextBuilder,
    conf,
    event::run,
};

pub mod io;
pub mod ext;
pub mod game;

pub mod util {
    use ggez::graphics::Color;
    use ggez::{Context, input::keyboard::{self, KeyCode}};
    use nalgebra::base::coordinates::XY;
    pub type Vector2 = nalgebra::Vector2<f32>;
    pub type Point2 = nalgebra::Point2<f32>;
    pub type Rotation2 = nalgebra::Rotation2<f32>;

    pub const TRANS: Color = Color{r:1.,g:1.,b:1.,a:0.5};
    pub const GREEN: Color = Color{r:0.1,g:0.7,b:0.1,a:1.};
    pub const RED: Color = Color{r:1.,g:0.,b:0.,a:1.};
    pub const BLUE: Color = Color{r:0.,g:0.,b:1.,a:1.};

    /// Makes a unit vector from a given direction angle
    pub fn angle_to_vec(angle: f32) -> Vector2 {
        let (sin, cos) = angle.sin_cos();
        Vector2::new(cos, sin)
    }
    /// Gets the direction angle on the screen (0 is along the x-axis) of a vector
    pub fn angle_from_vec(v: Vector2) -> f32 {
        let XY{x, y} = *v;
        y.atan2(x)
    }

    pub fn ver(ctx: &Context) -> f32 {
        <f32>::from((keyboard::is_key_pressed(ctx, KeyCode::S) || keyboard::is_key_pressed(ctx, KeyCode::Down)) as i8 -
            (keyboard::is_key_pressed(ctx, KeyCode::W) || keyboard::is_key_pressed(ctx, KeyCode::Up)) as i8)
    }
    pub fn hor(ctx: &Context) -> f32 {
        <f32>::from((keyboard::is_key_pressed(ctx, KeyCode::D) || keyboard::is_key_pressed(ctx, KeyCode::Right)) as i8 -
            (keyboard::is_key_pressed(ctx, KeyCode::A) || keyboard::is_key_pressed(ctx, KeyCode::Left)) as i8)
    }
}

use self::game::Master;

fn main() {
    let mut args = args().skip(1);
    let arg = args.next().unwrap_or_default();

    // Set window mode
    let window_mode = conf::WindowMode::default().dimensions(1152., 648.);

    // Create a context (the part that runs the game loop)
    let (mut ctx, mut events) = ContextBuilder::new("cowfarg", "LFalch")
        .window_setup(conf::WindowSetup::default().title("Kofarve"))
        .window_mode(window_mode)
        .build().unwrap();

    #[cfg(debug_assertions)]
    {
        // Add the workspace directory to the filesystem when running with cargo
        use ggez::filesystem;
        if let Ok(manifest_dir) = ::std::env::var("CARGO_MANIFEST_DIR") {
            let mut path = ::std::path::PathBuf::from(manifest_dir);
            path.push("resources");
            filesystem::mount(&mut ctx, &path, true);
        }
    }

    match Master::new(&mut ctx, &arg) {
        Err(e) => {
            eprintln!("Couldn't load game {}", e);
        }
        Ok(mut game) => {
            match run(&mut ctx, &mut events, &mut game) {
                Ok(_) => (),
                Err(e) => eprintln!("Error occured: {}", e)
            }
        }
    }
}
