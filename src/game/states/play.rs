use crate::{
    util::{
        BLUE, GREEN, RED,
        angle_from_vec,
        ver, hor,
        Vector2, Point2
    },
    io::tex::PosText,
    game::{
        State, GameState, world::{World, Grid},
        event::Event
    },
};
use ggez::{
    Context, GameResult,
    graphics::{
        self, Drawable, DrawMode, Rect,
        MeshBuilder, Mesh,
        spritebatch::SpriteBatch,
    },
    input::mouse,
};

/// The state of the game
pub struct Play {
    top_text: PosText,
    status_text: PosText,
    hud: Hud,
    world: World,
    holes: SpriteBatch,
    cur_pickup: Option<usize>,
    victory_time: f32,
    time: usize,
}

impl Play {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, s: &mut State) -> GameResult<Box<dyn GameState>> {
        mouse::set_cursor_hidden(ctx, true);

        Ok(Box::new(
            Play {
                top_text: s.assets.text(Point2::new(4., 4.)).and_text("100"),
                status_text: s.assets.text(Point2::new(s.width as f32 / 2., s.height as f32 / 2. + 32.)).and_text(""),
                hud: Hud::new(ctx)?,
                time: 0,
                victory_time: 0.,
                cur_pickup: None,
                world: World { grid: Grid::new(16, 16) },
                holes: SpriteBatch::new(s.assets.get_img(ctx, "common/hole").clone()),
            }
        ))
    }
}

impl GameState for Play {
    #[allow(clippy::cognitive_complexity)]
    fn update(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        let mouse_pos = s.mouse - s.offset;
        // Define player velocity here already because enemies need it
        let camera_vel = Vector2::new(hor(&ctx), ver(&ctx));

        Ok(())
    }
    fn logic(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        let dist = s.mouse - s.offset; // - self.world.player.obj.pos;

        self.hud.update_bars(ctx)?;

        // Center the camera on the player
        // let p = self.world.player.obj.pos;
        // s.focus_on(p);
        Ok(())
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.world.grid.draw(ctx, &s.assets)?;

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.hud.draw(ctx)?;

        self.top_text.draw_text(ctx)?;
        self.status_text.draw_center(ctx)?;

        let drawparams = graphics::DrawParam {
            dest: s.mouse.into(),
            offset: Point2::new(0.5, 0.5).into(),
            color: RED,
            .. Default::default()
        };
        let img = s.assets.get_img(ctx, "common/crosshair");
        graphics::draw(ctx, &*img, drawparams)
    }
    fn event_up(&mut self, s: &mut State, ctx: &mut Context, event: Event) {
        // use self::KeyCode::*;
        match event {
            _ => (),
        }
    }

    fn get_world(&self) -> Option<&World> {
        Some(&self.world)
    }
    fn get_mut_world(&mut self) -> Option<&mut World> {
        Some(&mut self.world)
    }
}

#[derive(Debug)]
pub struct Hud {
    hud_bar: Mesh,
    hp_bar: Mesh,
    armour_bar: Mesh,
    loading_bar: Mesh,
}

impl Hud {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let hud_bar = MeshBuilder::new()
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 1., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 29., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 57., w: 102., h: 26.}, graphics::BLACK)
            .build(ctx)?;

        let hp_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 2., w: 0., h: 24.}, GREEN)?;
        let armour_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 30., w: 0., h: 24.}, BLUE)?;
        let loading_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 58., w: 0., h: 24.}, RED)?;

        Ok(Hud{
            hud_bar,
            hp_bar,
            armour_bar,
            loading_bar,
        })
    }
    pub fn update_bars(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.hp_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 2., w: 100., h: 24.}, GREEN)?;
        self.armour_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 30., w: 100., h: 24.}, BLUE)?;
        self.loading_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 58., w: 100., h: 24.}, RED)?;

        Ok(())
    }
    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        self.hud_bar.draw(ctx, Default::default())?;
        self.hp_bar.draw(ctx, Default::default())?;
        self.armour_bar.draw(ctx, Default::default())?;
        self.loading_bar.draw(ctx, Default::default())
    }
}