use std::path::{Path, PathBuf};
use crate::{
    util::{Vector2, Point2},
    ext::{MouseDown, InputState, Modifiers},
    io::{
        snd::MediaPlayer,
        tex::Assets,
    },
    obj::{health::Health, weapon::WeaponInstance},
};
use ggez::{
    Context, GameResult,
    graphics::{self, Matrix4},
    timer,
    event::{EventHandler, MouseButton, MouseState, Keycode, Mod}
};
use self::world::Level;

/// Stuff related to things in the world
pub mod world;
pub mod editor;
pub mod play;
pub mod menu;
pub mod lose;
pub mod win;

use self::menu::Menu;
use self::world::Statistics;

pub enum StateSwitch {
    Menu,
    Editor(Option<Level>),
    Play(Level),
    PlayWith{
        lvl: Box<Level>,
        health: Health,
        wep: Option<WeaponInstance<'static>>,
    },
    Lose(Box<Statistics>),
    Win(Box<Statistics>),
}

pub trait GameState {
    fn update(&mut self, _: &mut State, _: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn logic(&mut self, _: &mut State, _: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw(&mut self, _: &State, _: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw_hud(&mut self, _: &State, _: &mut Context) -> GameResult<()>;

    fn key_down(&mut self, _: &mut State, _: &mut Context, _: Keycode) {

    }
    fn key_up(&mut self, _: &mut State, _: &mut Context, _: Keycode) {

    }
    fn mouse_down(&mut self, _: &mut State, _: &mut Context, _: MouseButton) {

    }
    fn mouse_up(&mut self, _: &mut State, _: &mut Context, _: MouseButton) {

    }
}

pub struct Master {
    gs: Box<dyn GameState>,
    state: State,
}

pub enum Content {
    Campaign(Campaign),
    File(PathBuf),
    None
}

/// The state of the game
pub struct State {
    mouse_down: MouseDown,
    input: InputState,
    modifiers: Modifiers,
    assets: Assets,
    mplayer: MediaPlayer,
    width: u32,
    height: u32,
    mouse: Point2,
    offset: Vector2,
    switch_state: Option<StateSwitch>,
    content: Content,
}

const DESIRED_FPS: u32 = 60;

pub(crate) const DELTA: f32 = 1. / DESIRED_FPS as f32;

impl Master {
    #[allow(clippy::new_ret_no_self)]
    /// Make a new state object
    pub fn new(ctx: &mut Context, arg: &str) -> GameResult<Self> {
        // Background colour is black
        graphics::set_background_color(ctx, (33, 33, 255, 255).into());
        // Initialise assets
        let assets = Assets::new(ctx)?;
        let mplayer = MediaPlayer::new(ctx)?;

        // Get the window's dimensions
        let width = ctx.conf.window_mode.width;
        let height = ctx.conf.window_mode.height;

        let content;

        if arg.is_empty() {
            content = Content::None
        } else {
            content = Content::File(arg.to_owned().into())
        }

        let mut state = State {
            content,
            switch_state: None,
            input: Default::default(),
            mouse_down: Default::default(),
            modifiers: Default::default(),
            assets,
            mplayer,
            width,
            height,
            mouse: Point2::new(0., 0.),
            offset: Vector2::new(0., 0.),
        };

        Ok(Master {
            gs: Menu::new(ctx, &mut state)?,
            state,
        })
    }
}

impl State {
    /// Sets the offset so that the given point will be centered on the screen
    fn focus_on(&mut self, p: Point2) {
        self.offset = -p.coords + 0.5 * Vector2::new(self.width as f32, self.height as f32);
    }
    fn switch(&mut self, ss: StateSwitch) {
        self.switch_state = Some(ss);
    }
}

use std::mem;

impl EventHandler for Master {
    // Handle the game logic
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if let Some(gsb) = mem::replace(&mut self.state.switch_state, None) {
            use self::StateSwitch::*;
            self.gs = match gsb {
                PlayWith{lvl, health, wep} => play::Play::new(ctx, &mut self.state, *lvl, Some((health, wep))),
                Play(lvl) => play::Play::new(ctx, &mut self.state, lvl, None),
                Menu => menu::Menu::new(ctx, &mut self.state),
                Editor(l) => editor::Editor::new(ctx, &self.state, l),
                Win(stats) => win::Win::new(ctx, &mut self.state, *stats),
                Lose(stats) => lose::Lose::new(ctx, &mut self.state, *stats),
            }?;
        }

        // Run this for every 1/60 of a second has passed since last update
        // Can in theory become slow
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.gs.update(&mut self.state, ctx)?;
        }
        self.gs.logic(&mut self.state, ctx)
    }

    // Draws everything
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Clear the screen first
        graphics::clear(ctx);

        // Offset the current drawing with a translation from the `offset`
        graphics::push_transform(ctx, Some(Matrix4::new_translation(&self.state.offset.fixed_resize(0.))));
        graphics::apply_transformations(ctx)?;

        self.gs.draw(&self.state, ctx)?;

        // Pop the offset tranformation to draw the UI on the screen
        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx)?;

        self.gs.draw_hud(&self.state, ctx)?;

        // Flip the buffers to see what we just drew
        graphics::present(ctx);

        // Give the computer some time to do other things
        timer::yield_now();
        Ok(())
    }
    /// Handle key down events
    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _: Mod, repeat: bool) {
        // If this is a repeat event, we don't care
        if repeat {
            return
        }

        use self::Keycode::*;
        // Update input axes and quit game on Escape
        match keycode {
            W | Up => self.state.input.ver -= 1,
            S | Down => self.state.input.ver += 1,
            A | Left => self.state.input.hor -= 1,
            D | Right => self.state.input.hor += 1,
            LShift => self.state.modifiers.shift = true,
            LCtrl => self.state.modifiers.ctrl = true,
            LAlt => self.state.modifiers.alt = true,
            Escape => ctx.quit().unwrap(),
            _ => (),
        }
        self.gs.key_down(&mut self.state, ctx, keycode)
    }
    /// Handle key release events
    fn key_up_event(&mut self, ctx: &mut Context, keycode: Keycode, _: Mod, repeat: bool) {
        // Still don't care about repeats
        if repeat {
            return
        }

        use self::Keycode::*;
        match keycode {
            W | Up => self.state.input.ver += 1,
            S | Down => self.state.input.ver -= 1,
            A | Left => self.state.input.hor += 1,
            D | Right => self.state.input.hor -= 1,
            LShift => self.state.modifiers.shift = false,
            LCtrl => self.state.modifiers.ctrl = false,
            LAlt => self.state.modifiers.alt = false,
            _ => (),
        }
        self.gs.key_up(&mut self.state, ctx, keycode)
    }
    /// Handle mouse down event
    fn mouse_button_down_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: i32, _y: i32) {
        use self::MouseButton::*;
        match btn {
            Left => self.state.mouse_down.left = true,
            Middle => self.state.mouse_down.middle = true,
            Right => self.state.mouse_down.right = true,
            _ => ()
        }
        self.gs.mouse_down(&mut self.state, ctx, btn)
    }
    /// Handle mouse release events
    fn mouse_button_up_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: i32, _y: i32) {
        use self::MouseButton::*;
        match btn {
            Left => self.state.mouse_down.left = false,
            Middle => self.state.mouse_down.middle = false,
            Right => self.state.mouse_down.right = false,
            _ => ()
        }
        self.gs.mouse_up(&mut self.state, ctx, btn)
    }
    /// Handles mouse movement events
    fn mouse_motion_event(&mut self, _: &mut Context, _: MouseState, x: i32, y: i32, _: i32, _: i32) {
        self.state.mouse = Point2::new(x as f32, y as f32);
    }
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        false
    }
}


use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct Campaign {
    pub levels: Vec<Level>,
    pub current: usize,
}

impl Campaign {
    pub fn load<P: AsRef<Path>>(p: P) -> GameResult<Self> {
        let file = BufReader::new(File::open(p)?);

        let mut levels = Vec::new();

        for line in file.lines() {
            let line = line?;
            levels.push(Level::load(&line.trim())?);
        }

        Ok(Campaign {
            levels,
            current: 0,
        })
    }
    pub fn next_level(&mut self) -> Option<Level> {
        let ret = self.levels.get(self.current).cloned();
        self.current += 1;
        ret
    }
}
