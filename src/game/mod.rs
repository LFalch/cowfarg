use std::sync::Mutex;
use std::fmt::{self, Display};
use crate::{
    util::{Vector2, Point2, RED, GREEN, BLUE},
    io::{
        snd::MediaPlayer,
        tex::{Assets, PosText},
    },
};
use ggez::{
    nalgebra::Matrix4,
    Context, GameResult,
    graphics::{self, DrawMode, Rect, Mesh, Text, TextFragment, DrawParam, Color},
    timer,
    input::mouse::{self, MouseCursor},
    event::EventHandler
};
use clipboard::{ClipboardContext, ClipboardProvider};
use log::{Log, Metadata, Record, Level as LogLevel};
use lazy_static::lazy_static;

/// Stuff related to things in the world
pub mod world;
pub mod states;

use self::states::menu::Menu;

pub enum StateSwitch {
    Menu,
    Play,
}

pub mod event {
    pub use ggez::event::{MouseButton, KeyCode, KeyMods};
    pub enum Event {
        Key(KeyCode),
        Mouse(MouseButton),
    }
}

use event::*;

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
    fn event_down(&mut self, _: &mut State, _: &mut Context, _: Event) { }
    fn event_up(&mut self, _: &mut State, _: &mut Context, _: Event) { }

    fn get_world(&self) -> Option<&world::World> {
        None
    }
    fn get_mut_world(&mut self) -> Option<&mut world::World> {
        None
    }
}

lazy_static! {
    static ref CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger::default();
}

#[derive(Debug, Default)]
struct ConsoleLogger {
    fragments: Mutex<Vec<TextFragment>>
}

impl ConsoleLogger {
    #[inline]
    fn get_colour(l: LogLevel) -> Option<Color> {
        use self::LogLevel::*;
        match l {
            Trace => Some(BLUE),
            Info => None,
            Debug => Some(GREEN),
            Warn => Some(Color{r:1.,g:1.,b:0.,a:1.}),
            Error => Some(RED),
        }
    }
    fn empty(&self) -> impl Iterator<Item=TextFragment> {
        let frags = std::mem::replace(&mut *self.fragments.lock().unwrap(), Vec::new());
        
        frags.into_iter()
    }
}

impl Log for ConsoleLogger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Only want to deal with logs from this crate
        metadata.target().starts_with("cowfarg")
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            print!("{}: ", record.level());

            println!("{}", record.args());

            let frag: TextFragment = format!("{}\n", record.args()).into();

            let mut frags = self.fragments.lock().unwrap();

            if let Some(color) = Self::get_colour(record.level()) {
                frags.push(frag.color(color));
            } else {
                frags.push(frag);
            }
        }
    }
    fn flush(&self) {}
}

const PROMPT_Y: f32 = 196.;

#[derive(Debug)]
pub struct Console {
    history: Text,
    prompt: PosText,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CommandError {
    NoWorld,
    NoCampaign,
    InvalidArg,
    NoSuchLevel,
    NoSuchWeapon,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CommandError::*;
        match *self {
            NoWorld => "No world".fmt(f),
            NoCampaign => "No campaign loaded".fmt(f),
            InvalidArg => "Invalid argument".fmt(f),
            NoSuchLevel => "No such level".fmt(f),
            NoSuchWeapon => "No such weapon".fmt(f),
        }
    }
}

impl Console {
    fn new(_ctx: &mut Context, assets: &Assets) -> GameResult<Self> {
        log::set_logger(&*CONSOLE_LOGGER).expect("to be first logger");
        log::set_max_level(log::LevelFilter::Trace);

        Ok(Console {
            history: assets.raw_text_with("Acheivements disabled.\n", 18.),
            prompt: assets.text(Point2::new(0., PROMPT_Y)).and_text("> ").and_text(String::with_capacity(32)),
        })
    }
    fn execute(&mut self, ctx: &mut Context, state: &mut State, gs: &mut dyn GameState) -> GameResult<()> {
        let prompt = &mut self.prompt.text.fragments_mut()[1].text;

        self.history.add(format!("> {}\n", prompt));

        let cap = prompt.capacity();
        let prompt = mem::replace(prompt, String::with_capacity(cap));
        let args: Vec<_> = prompt.split(<char>::is_whitespace).collect();
        
        if let Err(s) = self.handle(ctx, state, gs, args) {
            error!("{}", s);
        }

        Ok(())
    }
    fn handle(&mut self, ctx: &mut Context, state: &mut State, gs: &mut dyn GameState, args: Vec<&str>) -> Result<(), CommandError> {
        // use self::CommandError::*;

        match args[0] {
            "" => (),
            "clear" => self.history = state.assets.raw_text_with("", 18.),
            "reload" => {
                state.switch(StateSwitch::Menu);
            }
            "hello" => {
                info!("Hello!");
            },
            "quit" => {
                ctx.continuing = false;
            }
            cmd => {
                warn!("  Unknown command `{}'!", cmd);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConsoleStatus {
    Open {
        cursor: MouseCursor,
        cursor_hidden: bool,
    },
    Closed
}

impl ConsoleStatus {
    pub fn is_open(self) -> bool {
        if let ConsoleStatus::Open{..} = self {
            true
        } else {
            false
        }
    }
    pub fn open(&mut self, ctx: &Context) {
        if let ConsoleStatus::Closed = self {
            *self = ConsoleStatus::Open{
                cursor: mouse::cursor_type(ctx),
                cursor_hidden: mouse::cursor_hidden(ctx),
            };
        }
    }
    pub fn close(&mut self, ctx: &mut Context) {
        if let ConsoleStatus::Open{cursor, cursor_hidden} = std::mem::replace(self, ConsoleStatus::Closed) {
            mouse::set_cursor_type(ctx, cursor);
            mouse::set_cursor_hidden(ctx, cursor_hidden);
        }
    }
}

pub struct Master {
    gs: Box<dyn GameState>,
    state: State,
    console_status: ConsoleStatus,
    console: Console,
}

/// The state of the game
pub struct State {
    assets: Assets,
    mplayer: MediaPlayer,
    width: f32,
    height: f32,
    mouse: Point2,
    offset: Vector2,
    switch_state: Option<StateSwitch>,
}

const DESIRED_FPS: u32 = 60;

pub(crate) const DELTA: f32 = 1. / DESIRED_FPS as f32;

impl Master {
    /// Make a new state object
    pub fn new(ctx: &mut Context, arg: &str) -> GameResult<Self> {
        // Initialise assets
        let assets = Assets::new(ctx)?;
        let mut mplayer = MediaPlayer::new();
        mplayer.register_music(ctx, "music", true)?;
        mplayer.register_music(ctx, "victory", false)?;

        // Get the window's dimensions
        let Rect {w: width, h: height, ..} = graphics::screen_coordinates(ctx);

        let mut state = State {
            switch_state: None,
            assets,
            mplayer,
            width,
            height,
            mouse: Point2::new(0., 0.),
            offset: Vector2::new(0., 0.),
        };

        Ok(Master {
            console: Console::new(ctx, &state.assets)?,
            console_status: ConsoleStatus::Closed,
            gs: Menu::new(ctx, &mut state)?,
            state,
        })
    }
}

impl State {
    /// Sets the offset so that the given point will be centered on the screen
    fn focus_on(&mut self, p: Point2) {
        self.offset = -p.coords + 0.5 * Vector2::new(self.width, self.height);
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
            mouse::set_cursor_hidden(ctx, false);
            mouse::set_cursor_type(ctx, MouseCursor::Default);

            use self::StateSwitch::*;
            self.gs = match gsb {
                Play => states::play::Play::new(ctx, &mut self.state),
                Menu => states::menu::Menu::new(ctx, &mut self.state),
            }?;
        }
        if self.console_status.is_open() {
            while timer::check_update_time(ctx, DESIRED_FPS) {}

            for frag in CONSOLE_LOGGER.empty() {
                self.console.history.add(frag);
            }
            while self.console.history.height(ctx) > PROMPT_Y as u32 {
                let new_history = self.console.history.fragments().iter().skip(1).cloned().fold(self.state.assets.raw_text(18.), |mut text, f| {
                    text.add(f);
                    text
                });
                self.console.history = new_history;
            }

            Ok(())
        } else {

            // Run this for every 1/60 of a second has passed since last update
            // Can in theory become slow
            while timer::check_update_time(ctx, DESIRED_FPS) {
                self.gs.update(&mut self.state, ctx)?;
            }
            self.gs.logic(&mut self.state, ctx)
        }
    }

    // Draws everything
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Clear the screen first
        graphics::clear(ctx, (33, 33, 255, 255).into());

        // Offset the current drawing with a translation from the `offset`
        graphics::push_transform(ctx, Some(Matrix4::new_translation(&self.state.offset.fixed_resize(0.))));
        graphics::apply_transformations(ctx)?;

        self.gs.draw(&self.state, ctx)?;

        // Pop the offset tranformation to draw the UI on the screen
        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx)?;

        self.gs.draw_hud(&self.state, ctx)?;

        if self.console_status.is_open() {
            let console_bg = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect::new(0., 0., self.state.width as f32, self.state.height as f32 / 3.), graphics::BLACK)?;
            graphics::draw(ctx, &console_bg, DrawParam::new())?;


            graphics::draw(ctx, &self.console.history, DrawParam::default())?;
            self.console.prompt.draw_text(ctx)?;
        }

        // Flip the buffers to see what we just drew
        graphics::present(ctx)?;

        // Give the computer some time to do other things
        timer::yield_now();
        Ok(())
    }
    /// Handle key down events
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, km: KeyMods, repeat: bool) {
        // If this is a repeat event, we don't care
        if repeat {
            return
        }

        use self::KeyCode::*;
        match keycode {
            Escape if km.contains(KeyMods::SHIFT) => ctx.continuing = false,
            keycode if !self.console_status.is_open() => self.gs.event_down(&mut self.state, ctx, Event::Key(keycode)),
            _ => (),
        }
    }
    /// Handle key release events
    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods) {
        if !self.console_status.is_open() {
            match keycode {
                KeyCode::Tab => self.console_status.open(ctx),
                keycode => self.gs.event_up(&mut self.state, ctx, Event::Key(keycode))
            }
        }
    }
    /// Handle mouse down event
    fn mouse_button_down_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: f32, _y: f32) {
        if !self.console_status.is_open() {
            self.gs.event_down(&mut self.state, ctx, Event::Mouse(btn))
        }
    }
    /// Handle mouse release events
    fn mouse_button_up_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: f32, _y: f32) {
        if !self.console_status.is_open() {
            self.gs.event_up(&mut self.state, ctx, Event::Mouse(btn))
        }
    }
    fn text_input_event(&mut self, ctx: &mut Context, c: char) {
        if self.console_status.is_open() {
            if c.is_control() {
                match c {
                    // Backspace
                    '\u{8}' => {self.console.prompt.text.fragments_mut()[1].text.pop();},
                    // Delete
                    '\u{7f}' => (),
                    // Escape
                    '\u{1b}' => self.console_status.close(ctx),
                    '\t' => {
                        // Do tab completion
                    }
                    // Paste
                    '\u{16}' => {
                        let mut cc = ClipboardContext::new().unwrap();
                        let to_paste: String = cc.get_contents().unwrap();

                        self.console.prompt.text.fragments_mut()[1].text.push_str(&to_paste);
                    },
                    // Return (note sure whether newline is used on other platforms, so handling it in key_up)
                    '\r' => self.console.execute(ctx, &mut self.state, &mut *self.gs).unwrap(),
                    c => {self.console.history.add(format!("Unknown control character {:?}\n", c));}
                }
            } else {
                self.console.prompt.text.fragments_mut()[1].text.push(c);
            }
        }
    }
    /// Handles mouse movement events
    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _: f32, _: f32) {
        self.state.mouse = Point2::new(x, y);
        if let ConsoleStatus::Open{cursor, cursor_hidden} = self.console_status {
            if y > PROMPT_Y {
                mouse::set_cursor_type(ctx, cursor);
                mouse::set_cursor_hidden(ctx, cursor_hidden);
            } else {
                mouse::set_cursor_type(ctx, MouseCursor::Default);
                mouse::set_cursor_hidden(ctx, false);
            }
        }
    }
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        false
    }
}
