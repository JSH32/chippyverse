use std::{fs, sync::Arc};

use crate::{
    input::{InputHandler, KeyEvent},
    window::{self, Window, WindowContainer},
};
use chippy_core::ExecutingChip8;
use egui::{Image, TextureId, Vec2};
use mq::{Texture, TextureParams};

use crate::debugger::DebuggerWindow;
use {egui_miniquad as egui_mq, miniquad as mq};

pub struct MainApp {
    chip8: Arc<ExecutingChip8>,
    screen_texture: Option<Texture>,
    debugger_window: WindowContainer<DebuggerWindow>,
}

impl MainApp {
    pub fn new() -> Self {
        let chip8 = Arc::new(ExecutingChip8::new());

        chip8
            .write()
            .unwrap()
            .load_rom(include_bytes!("Instruction-test.ch8").to_vec());

        chip8.set_running(true);

        let chip8_clone = chip8.clone();
        Self {
            chip8,
            screen_texture: None,
            debugger_window: WindowContainer::new(DebuggerWindow::new(chip8_clone)),
        }
    }

    fn screen_rgba(&self) -> [u8; 64 * 32 * 4] {
        let binding = self.chip8.read().unwrap();
        let screen_flattened = binding.screen.flatten();
        let mut buffer = [0; 64 * 32 * 4];

        // TODO: Make colors configurable for both foreground and background.
        for (i, el) in screen_flattened.iter().enumerate() {
            if *el {
                buffer[i * 4] = 255;
                buffer[i * 4 + 1] = 255;
                buffer[i * 4 + 2] = 255;
            } else {
                buffer[i * 4] = 0;
                buffer[i * 4 + 1] = 0;
                buffer[i * 4 + 2] = 0;
            }

            // Alpha always 100%
            buffer[i * 4 + 3] = 255;
        }

        buffer
    }
}

impl Window for MainApp {
    fn config(&self) -> mq::conf::Conf {
        mq::conf::Conf {
            high_dpi: true,
            icon: None,
            window_width: 640,
            window_height: 400,
            window_title: window::window_title("Emulator"),
            window_resizable: true,
            ..Default::default()
        }
    }

    fn on_open(&mut self, ctx: &mut mq::Context) {
        self.screen_texture = Some(Texture::from_data_and_format(
            ctx,
            vec![0; 64 * 32 * 4].as_slice(),
            TextureParams {
                format: mq::TextureFormat::RGBA8,
                wrap: mq::TextureWrap::Clamp,
                filter: mq::FilterMode::Nearest,
                width: 64,
                height: 32,
            },
        ));
    }

    fn update(&mut self, mq_ctx: &mut mq::Context) {
        self.screen_texture
            .unwrap()
            .update(mq_ctx, &self.screen_rgba());
    }

    fn draw(&mut self, mq_ctx: &mut mq::Context, egui_ctx: &mut egui_mq::EguiMq) {
        mq_ctx.clear(Some((1., 1., 1., 1.)), None, None);
        mq_ctx.begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        mq_ctx.end_render_pass();

        egui_ctx.run(mq_ctx, |_mq_ctx, egui_ctx| {
            egui::TopBottomPanel::top("my_panel").show(&egui_ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open ROM").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.chip8
                                    .write()
                                    .unwrap()
                                    .load_rom(fs::read(path).expect("Unable to read ROM"));
                            }
                        }

                        if ui
                            .add_enabled(
                                !self.debugger_window.is_open(),
                                egui::widgets::Button::new("Debugger"),
                            )
                            .clicked()
                        {
                            let _ = self.debugger_window.open();
                        }
                    });
                });
            });

            egui::CentralPanel::default().show(&egui_ctx, |ui| {
                ui.add_sized(
                    ui.available_size(),
                    Image::new(
                        TextureId::User(self.screen_texture.unwrap().gl_internal_id() as u64),
                        Vec2::new(640.0, 360.0),
                    ),
                );
            });
        });

        // Draw things behind egui here

        egui_ctx.draw(mq_ctx);

        // Draw things in front of egui here

        mq_ctx.commit_frame();
    }

    fn on_event(&mut self, _ctx: &mut mq::Context, event: window::Event) -> bool {
        match event {
            window::Event::KeyUp {
                keycode,
                keymods: _,
            } => self
                .chip8
                .write()
                .unwrap()
                .keypad
                .key_event(KeyEvent::KeyUp, keycode),
            window::Event::KeyDown {
                keycode,
                keymods: _,
                repeat,
            } => {
                if !repeat {
                    self.chip8
                        .write()
                        .unwrap()
                        .keypad
                        .key_event(KeyEvent::KeyDown, keycode)
                } else {
                    false
                }
            }
            _ => true,
        }
    }
}
