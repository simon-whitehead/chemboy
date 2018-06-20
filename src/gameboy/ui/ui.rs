use std;
use std::marker::PhantomData;

use conrod;
use conrod::{widget, Colorable, Labelable, Positionable, Sizeable, UiCell, Widget};
use conrod::position::{Align, Relative};
use find_folder;
use gfx_device_gl;
use gfx_core;
use gfx_core::factory::Factory;
use piston_window::*;
use piston_window::{PistonWindow, UpdateEvent, Window, WindowSettings};
use piston_window::{Flip, G2d, G2dTexture, Texture, TextureSettings};
use piston_window::texture::{Format, UpdateTexture};

use gameboy::ui::theme::Theme;
use gameboy::ui::ui_event::UIEvent;

widget_ids! {
    pub struct Ids {
        master_canvas,
        left_canvas,
        center_canvas,
        right_canvas,

        // Theme switcher
        theme_switcher_label,
        theme_switcher,

        // Disassembly
        disassembly_list
    }
}

pub struct Ui<'a> {
    conrod_ui: conrod::Ui,
    width: f64,
    height: f64,
    ids: Ids,
    text_vertex_data: Vec<u8>,
    glyph_cache: conrod::text::GlyphCache<'a>,
    text_texture: Texture<gfx_device_gl::Resources>,
    image_map: conrod::image::Map<Texture<gfx_device_gl::Resources>>,

    selected_theme: Option<usize>,
    dasm: Vec<String>,
}

impl<'a> Ui<'a> {
    pub fn new<F>(width: f64, height: f64, mut factory: F, rom: &[u8]) -> Ui
    where
        F: gfx_core::Factory<gfx_device_gl::Resources>,
    {
        let mut ui = conrod::UiBuilder::new([width, height])
            .theme(Self::base_theme())
            .build();

        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let font_path = assets.join("fonts/DejaVuSansMono.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        let (mut glyph_cache, mut text_texture_cache) = {
            const SCALE_TOLERANCE: f32 = 1.0;
            const POSITION_TOLERANCE: f32 = 1.0;
            let cache = conrod::text::GlyphCache::new(
                width as u32,
                height as u32,
                SCALE_TOLERANCE,
                POSITION_TOLERANCE,
            );
            let buffer_len = width as usize * height as usize;
            let init = vec![128; buffer_len];
            let settings = TextureSettings::new();
            let factory = &mut factory;
            let texture = G2dTexture::from_memory_alpha(
                factory,
                &init,
                width as u32,
                height as u32,
                &settings,
            ).unwrap();
            (cache, texture)
        };

        let ids = Ids::new(ui.widget_id_generator());
        let mut bg_color = conrod::color::LIGHT_BLUE;
        let dasm = ::gameboy::disassembler::disassemble(rom);

        Ui {
            conrod_ui: ui,
            width: width,
            height: height,
            ids: ids,
            text_vertex_data: Vec::new(),
            glyph_cache: glyph_cache,
            text_texture: text_texture_cache,
            image_map: conrod::image::Map::new(),
            selected_theme: Some(0),
            dasm: dasm,
        }
    }

    pub fn handle_event(&mut self, e: &Event) -> UIEvent {
        let mut result = UIEvent::None;
        // Convert the piston event to a conrod event.
        let (win_w, win_h) = (self.width as conrod::Scalar, self.height as conrod::Scalar);
        if let Some(evt) = conrod::backend::piston::event::convert(e.clone(), win_w, win_h) {
            self.conrod_ui.handle_event(evt);
        }

        e.update(|_| {
            let mut ui = self.conrod_ui.set_widgets();
            conrod::widget::Canvas::new()
                .flow_right(&[
                    (
                        self.ids.left_canvas,
                        conrod::widget::Canvas::new().w_h(410.0, win_h).pad(25.0),
                    ),
                    (
                        self.ids.center_canvas,
                        conrod::widget::Canvas::new().w_h(410.0, win_h).pad(25.0),
                    ),
                    (
                        self.ids.right_canvas,
                        conrod::widget::Canvas::new().w_h(410.0, win_h).pad(25.0),
                    ),
                ])
                .set(self.ids.master_canvas, &mut ui);

            // conrod::widget::Text::new("Theme: ")
            // .top_left_of(self.ids.left_canvas)
            // .set(self.ids.theme_switcher_label, &mut ui);
            //
            // let themes = vec!["Default", "Classic Gameboy"];
            // for selected_theme in conrod::widget::DropDownList::new(&themes, self.selected_theme)
            // .right_from(self.ids.theme_switcher_label, 10.0)
            // .w_h(250.0, 25.0)
            // .color(conrod::color::DARK_BLUE)
            // .label_color(conrod::color::WHITE)
            // .label("Theme")
            // .set(self.ids.theme_switcher, &mut ui) {
            // self.selected_theme = Some(selected_theme);
            // match self.selected_theme.unwrap() {
            // 0 => result = UIEvent::ThemeSwitched(Theme::Default),
            // _ => result = UIEvent::ThemeSwitched(Theme::ClassicDMG),
            // }
            // }

            let (mut items, scrollbar) = widget::List::flow_down(self.dasm.len())
                .mid_top_of(self.ids.left_canvas)
                .item_size(20.0)
                .scroll_kids_vertically()
                .scrollbar_on_top()
                .scrollbar_color(conrod::color::GREEN)
                .wh_of(self.ids.left_canvas)
                //.wh_of(self.ids.left_canvas)
                .set(self.ids.disassembly_list, &mut ui);

            while let Some(item) = items.next(&mut ui) {
                let i = item.i;
                let toggle = widget::Toggle::new(true)
                    .label(&self.dasm[i])
                    .label_x(Relative::Align(Align::Start))
                    .label_color(conrod::color::GREEN)
                    .label_font_size(10)
                    .color(conrod::color::BLACK);
                for v in item.set(toggle, &mut ui) {
                    // self.dasm[i] = v;
                }
            }

            if let Some(s) = scrollbar {
                s.set(&mut ui)
            }
        });

        result
    }

    pub fn draw(&mut self, c: conrod::backend::piston::draw::Context, g: &mut G2d) {
        if let Some(primitives) = self.conrod_ui.draw_if_changed() {
            // A function used for caching glyphs to the texture cache.
            let cache_queued_glyphs = |graphics: &mut G2d,
                                       cache: &mut G2dTexture,
                                       rect: conrod::text::rt::Rect<u32>,
                                       data: &[u8]| {
                let offset = [rect.min.x, rect.min.y];
                let size = [rect.width(), rect.height()];
                let format = Format::Rgba8;
                let encoder = &mut graphics.encoder;
                let text_vertex_data: Vec<_> =
                    data.iter().flat_map(|&b| vec![255, 255, 255, b]).collect();
                UpdateTexture::update(cache, encoder, format, &text_vertex_data[..], offset, size)
                    .expect("failed to update texture")
            };

            conrod::backend::piston::draw::primitives(
                primitives,
                c,
                g,
                &mut self.text_texture,
                &mut self.glyph_cache,
                &self.image_map,
                cache_queued_glyphs,
                |img| img,
            )
        }
    }

    pub fn base_theme() -> conrod::Theme {
        use conrod::position::{Align, Direction, Padding, Position, Relative};
        conrod::Theme {
            name: "chemboy theme".to_string(),
            padding: Padding::none(),
            x_position: Position::Relative(Relative::Align(Align::Start), None),
            y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
            background_color: conrod::color::BLACK,
            shape_color: conrod::color::LIGHT_CHARCOAL,
            border_color: conrod::color::BLACK,
            border_width: 0.0,
            label_color: conrod::color::WHITE,
            font_id: None,
            font_size_large: 18,
            font_size_medium: 16,
            font_size_small: 12,
            widget_styling: conrod::theme::StyleMap::default(),
            mouse_drag_threshold: 0.0,
            double_click_threshold: std::time::Duration::from_millis(500),
        }
    }
}
