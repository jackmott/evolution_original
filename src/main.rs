extern crate ggez;

mod apt;
mod imgui_wrapper;

use crate::imgui_wrapper::ImGuiWrapper;
use apt::*;
use simdeez::*;
use simdeez::avx2::*;
use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};

struct MainState<S:Simd> {
    pos_x: f32,
    imgui_wrapper: ImGuiWrapper,
    hidpi_factor: f32,
    pic: MonoPic<S>,
    img: graphics::Image,
}

impl<S:Simd> MainState<S> {
    fn new(mut ctx: &mut Context, hidpi_factor: f32) -> GameResult<MainState<S>> {
        let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
        let pic = MonoPic::new(10);
        let img = graphics::Image::from_rgba8(ctx,500,500, &pic.get_rgba8(500,500)[0..]).unwrap();
        let s = MainState {
            pos_x: 0.0,
            imgui_wrapper,
            hidpi_factor,
            pic,
            img,
        };
        Ok(s)
    }
}

impl<S:Simd> EventHandler for MainState<S> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        let _ = graphics::draw(ctx,&self.img,graphics::DrawParam::default());
        // Render game stuff
        {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                na::Point2::new(self.pos_x, 380.0),
                100.0,
                2.0,
                graphics::WHITE,
            )?;
            graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        }

        // Render game ui
        {
            self.imgui_wrapper.render(ctx, self.hidpi_factor);
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((
            button == MouseButton::Left,
            button == MouseButton::Right,
            button == MouseButton::Middle,
        ));
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((false, false, false));
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::P => (),
            _ => (),
        }
    }

    fn text_input_event(&mut self, _ctx: &mut Context, ch: char) {
        self.imgui_wrapper.update_keyboard(ch);
    }
}

pub fn main() -> ggez::GameResult {
    let hidpi_factor: f32;
    {
        // Create a dummy window so we can get monitor scaling information
        let cb = ggez::ContextBuilder::new("", "");
        let (_ctx, events_loop) = &mut cb.build()?;
        hidpi_factor = events_loop.get_primary_monitor().get_hidpi_factor() as f32;
        println!("main hidpi_factor = {}", hidpi_factor);
    }

    let cb = ggez::ContextBuilder::new("super_simple with imgui", "ggez")
        .window_setup(conf::WindowSetup::default().title("super_simple with imgui"))
        .window_mode(conf::WindowMode::default().dimensions(750.0, 500.0));
    let (ref mut ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::<Avx2>::new(ctx, hidpi_factor)?;

    event::run(ctx, event_loop, state)
}
