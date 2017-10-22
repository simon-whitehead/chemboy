use std;
use std::marker::PhantomData;

use conrod;
use conrod::{Colorable, Labelable, Positionable, Sizeable, UiCell, Widget};
use find_folder;
use gfx_device_gl;
use gfx_core;
use gfx_core::factory::Factory;
use piston_window::*;
use piston_window::{PistonWindow, UpdateEvent, Window, WindowSettings};
use piston_window::{Flip, G2d, G2dTexture, Texture, TextureSettings};
use piston_window::texture::{Format, UpdateTexture};

widget_ids! {
    pub struct Ids {
        canvas,
        test_button,
        introduction
    }
}

pub struct Ui {
    conrod_ui: conrod::Ui,
    width: f64,
    height: f64,
    ids: Ids,
    text_vertex_data: Vec<u8>,
    glyph_cache: conrod::text::GlyphCache,
    text_texture: Texture<gfx_device_gl::Resources>,
    image_map: conrod::image::Map<Texture<gfx_device_gl::Resources>>,
}

impl Ui {
    pub fn new<F>(width: f64, height: f64, mut factory: F) -> Ui
        where F: gfx_core::Factory<gfx_device_gl::Resources>
    {
        let mut ui = conrod::UiBuilder::new([width, height])
            .theme(Self::base_theme())
            .build();

        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        let (mut glyph_cache, mut text_texture_cache) = {
            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;
            let cache = conrod::text::GlyphCache::new(width as u32,
                                                      height as u32,
                                                      SCALE_TOLERANCE,
                                                      POSITION_TOLERANCE);
            let buffer_len = width as usize * height as usize;
            let init = vec![128; buffer_len];
            let settings = TextureSettings::new();
            let factory = &mut factory;
            let texture = G2dTexture::from_memory_alpha(factory,
                                                        &init,
                                                        width as u32,
                                                        height as u32,
                                                        &settings)
                .unwrap();
            (cache, texture)
        };

        let ids = Ids::new(ui.widget_id_generator());
        let mut bg_color = conrod::color::LIGHT_BLUE;

        Ui {
            conrod_ui: ui,
            width: width,
            height: height,
            ids: ids,
            text_vertex_data: Vec::new(),
            glyph_cache: glyph_cache,
            text_texture: text_texture_cache,
            image_map: conrod::image::Map::new(),
        }
    }

    pub fn handle_event(&mut self, e: &Event) {
        // Convert the piston event to a conrod event.
        let (win_w, win_h) = (self.width as conrod::Scalar, self.height as conrod::Scalar);
        if let Some(evt) = conrod::backend::piston::event::convert(e.clone(), win_w, win_h) {
            self.conrod_ui.handle_event(evt);
        }

        e.update(|_| {
            let mut ui = self.conrod_ui.set_widgets();
            conrod::widget::Canvas::new()
                .pad(30.0)
                .color(conrod::color::LIGHT_BLUE)
                .w_h(500.0, 500.0)
                .set(self.ids.canvas, &mut ui);
            const INTRODUCTION: &'static str =
                "This example aims to demonstrate all widgets that are provided by conrod.\n\nThe \
                 widget that you are currently looking at is the Text widget. The Text widget is \
                 one of several special \"primitive\" widget types which are used to construct \
                 all other widget types. These types are \"special\" in the sense that conrod \
                 knows how to render them via `conrod::render::Primitive`s.\n\nScroll down to see \
                 more widgets!";
            conrod::widget::Text::new(INTRODUCTION)
                .padded_w_of(self.ids.canvas, 20.0)
                .down(60.0)
                .align_middle_x_of(self.ids.canvas)
                .center_justify()
                .line_spacing(5.0)
                .set(self.ids.introduction, &mut ui);
            if conrod::widget::Button::new()
                .label("Click me")
                .middle_of(self.ids.canvas)
                .w_h(130.0, 130.0)
                .set(self.ids.test_button, &mut ui)
                .was_clicked() {
                println!("Clicked!");
            }
        });
    }

    pub fn draw<G>(&mut self, c: conrod::backend::piston::draw::Context, g: &mut G)
        where G: Graphics<Texture = Texture<gfx_device_gl::Resources>>
    {
        if let Some(primitives) = self.conrod_ui.draw_if_changed() {
            // A function used for caching glyphs to the texture cache.
            let cache_queued_glyphs = |graphics: &mut G,
                                       cache: &mut G::Texture,
                                       rect: conrod::text::rt::Rect<u32>,
                                       data: &[u8]| {
            };
            // Specify how to get the drawable texture from the image. In this case, the image
            // *is* the texture.
            fn texture_from_image<A>(img: &A) -> &A {
                img
            }

            let draw_state = c.draw_state;
            println!("Draw state: {:?}", draw_state);

            // Draw the conrod `render::Primitives`.
            conrod::backend::piston::draw::primitives(primitives,
                                                      c,
                                                      g,
                                                      &mut self.text_texture,
                                                      &mut self.glyph_cache,
                                                      &self.image_map,
                                                      cache_queued_glyphs,
                                                      texture_from_image);
        }
    }

    pub fn base_theme() -> conrod::Theme {
        use conrod::position::{Align, Direction, Padding, Position, Relative};
        conrod::Theme {
            name: "chemboy theme".to_string(),
            padding: Padding::none(),
            x_position: Position::Relative(Relative::Align(Align::Start), None),
            y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
            background_color: conrod::color::GREEN,
            shape_color: conrod::color::LIGHT_CHARCOAL,
            border_color: conrod::color::BLACK,
            border_width: 0.0,
            label_color: conrod::color::WHITE,
            font_id: None,
            font_size_large: 26,
            font_size_medium: 18,
            font_size_small: 12,
            widget_styling: conrod::theme::StyleMap::default(),
            mouse_drag_threshold: 0.0,
            double_click_threshold: std::time::Duration::from_millis(500),
        }
    }
}
