use std::sync::Arc;

use chippy_core::{
    opcode::{extract_opcode_from_array, OpCode},
    ExecutingChip8,
};
use egui::{Align, Color32, RichText, Ui};

use crate::window::{self, Window};

use {egui_miniquad as egui_mq, miniquad as mq};

enum DebuggerTab {
    Registers,
    Dissasembly,
}

pub struct DebuggerWindow {
    chip8: Arc<ExecutingChip8>,
    selected: DebuggerTab,
}

impl DebuggerWindow {
    pub fn new(chip8: Arc<ExecutingChip8>) -> Self {
        Self {
            chip8,
            selected: DebuggerTab::Registers,
        }
    }
}

impl Window for DebuggerWindow {
    fn draw(&mut self, ctx: &mut mq::Context, egui_ctx: &mut egui_mq::EguiMq) {
        ctx.clear(Some((1., 1., 1., 1.)), None, None);
        ctx.begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        ctx.end_render_pass();

        egui_ctx.run(ctx, |mq_ctx, egui_ctx| {
            egui::TopBottomPanel::top("debug_top").show(&egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new("Registers")
                                .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 0)),
                        )
                        .clicked()
                    {
                        self.selected = DebuggerTab::Registers;
                        mq_ctx.set_window_size(230, 500)
                    }

                    ui.separator();

                    if ui
                        .add(
                            egui::Button::new("Dissasembly")
                                .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 0)),
                        )
                        .clicked()
                    {
                        self.selected = DebuggerTab::Dissasembly;
                        mq_ctx.set_window_size(850, 540)
                    }

                    ui.separator();

                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(!self.chip8.is_running(), egui::Button::new("⮫"))
                            .clicked()
                        {
                            self.chip8.write().unwrap().interpreter()
                        }

                        if ui
                            .button(if self.chip8.is_running() {
                                "⏸"
                            } else {
                                "▶"
                            })
                            .clicked()
                        {
                            self.chip8.set_running(!self.chip8.is_running())
                        }
                    })
                })
            });

            egui::CentralPanel::default().show(&egui_ctx, |ui| {
                let chip8 = self.chip8.read().unwrap();

                egui::ScrollArea::vertical()
                    .hscroll(true)
                    .show(ui, |ui| match self.selected {
                        DebuggerTab::Registers => {
                            egui::Grid::new("debug_registers")
                                .num_columns(2)
                                .min_col_width(100.0)
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.heading("PC");
                                    ui.monospace(format!("{:X}", chip8.pc));
                                    ui.end_row();

                                    ui.heading("SP");
                                    ui.monospace(format!("{:X}", chip8.sp));
                                    ui.end_row();

                                    ui.heading("I");
                                    ui.monospace(format!("{:X}", chip8.index));
                                    ui.end_row();

                                    for v in 0..15 {
                                        ui.heading(format!("V{}", v));
                                        ui.monospace(format!("{:X}", chip8.registers[v]));
                                        ui.end_row();
                                    }

                                    ui.heading("DT");
                                    ui.monospace(format!("{:X}", chip8.delay_timer));
                                    ui.end_row();

                                    ui.heading("ST");
                                    ui.monospace(format!("{:X}", chip8.sound_timer));
                                    ui.end_row();
                                });
                        }
                        DebuggerTab::Dissasembly => {
                            egui::Grid::new("debug_dissasembly")
                                .num_columns(4)
                                .striped(true)
                                .min_col_width(120.0)
                                .show(ui, |ui| {
                                    ui.heading("Location");
                                    ui.heading("Value");
                                    ui.heading("Opcode");
                                    ui.heading("Description");
                                    ui.end_row();

                                    let opcode_row = |ui: &mut Ui, idx, current| {
                                        let value = chip8.memory[idx as usize];

                                        let opcode_str = OpCode::from_opcode(
                                            extract_opcode_from_array(&chip8.memory, idx as usize),
                                        )
                                        .get_opcode_str();

                                        ui.monospace(RichText::new(format!("{:X}", idx)).color(
                                            if current {
                                                Color32::LIGHT_RED
                                            } else {
                                                Color32::GRAY
                                            },
                                        ));

                                        ui.monospace(format!("{:X}", value));
                                        ui.monospace(format!("{}", opcode_str.0));
                                        ui.monospace(format!("{}", opcode_str.1));
                                        ui.end_row();
                                    };

                                    for i in chip8.pc - 11..chip8.pc {
                                        opcode_row(ui, i, false);
                                    }

                                    opcode_row(ui, chip8.pc, true);

                                    for i in chip8.pc..(chip8.pc + 10) {
                                        opcode_row(ui, i, false);
                                    }
                                });
                        }
                    });
            });
        });

        egui_ctx.draw(ctx);
        ctx.commit_frame();
    }

    fn update(&mut self, _ctx: &mut mq::Context) {}

    fn config(&self) -> mq::conf::Conf {
        mq::conf::Conf {
            high_dpi: true,
            window_width: 300,
            window_height: 500,
            icon: None,
            window_title: window::window_title("Debugger"),
            window_resizable: true,
            ..Default::default()
        }
    }
}
