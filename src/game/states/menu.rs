use crate::{
    util::Point2,
    io::{
        tex::PosText,
        btn::Button,
    },
    game::{
        State, GameState, StateSwitch,
        event::{Event::{self, Mouse}, MouseButton}
    },
};
use ggez::{
    Context, GameResult,
    graphics::Rect,
};

/// The state of the game
pub struct Menu {
    title_txt: PosText,
    buttons: Vec<Button<Callback>>,
    corner_text: Option<PosText>,
}

enum Callback {
    SwitchPlay,
}

// ↓
fn button_rect(w: f32, i: f32) -> Rect {
    Rect{x:3. * w / 7., y: 64. + i * 68., w:w / 7., h:64.}
}

impl Menu {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, s: &mut State) -> GameResult<Box<dyn GameState>> {
        let w = s.width as f32;

        let corner_text = None;
        s.mplayer.play(ctx, "music")?;

        let buttons = vec![Button::new(ctx, &s.assets, button_rect(w, 0.), "Play", Callback::SwitchPlay)?];

        Ok(Box::new(Menu {
            title_txt: s.assets.text_sized(Point2::new(w / 2., 16.), 32.).and_text("Main Menu"),
            buttons,
            corner_text,
        }))
    }
}

impl GameState for Menu {
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        self.title_txt.draw_center(ctx)?;
        if let Some(ref txt) = self.corner_text {
            txt.draw_text(ctx)?;
        }
        for button in &self.buttons {
            button.draw(ctx)?;
        }
        Ok(())
    }
    fn event_up(&mut self, s: &mut State, ctx: &mut Context, event: Event) {
        if let Mouse(MouseButton::Left) = event {
            for button in &self.buttons {
                if button.in_bounds(s.mouse) {
                    s.mplayer.stop(ctx, "music").unwrap();
                    match &button.callback {
                        Callback::SwitchPlay => {
                            s.switch(StateSwitch::Play);
                        },
                    }
                }
            }
        }
    }
}
