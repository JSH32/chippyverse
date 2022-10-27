use {egui_miniquad as egui_mq, miniquad as mq};

use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use egui::mutex::RwLock;

/// Internal wrapper window for handling miniquad events.
struct InternalWindow<T: Window> {
    /// Window wrapper object
    window: Arc<RwLock<T>>,
    running: Arc<AtomicBool>,
    egui_ctx: egui_mq::EguiMq,
}

impl<T: Window> InternalWindow<T> {
    fn new(ctx: &mut mq::Context, running: Arc<AtomicBool>, window: Arc<RwLock<T>>) -> Self {
        window.write().on_open(ctx);

        Self {
            window,
            running,
            egui_ctx: egui_mq::EguiMq::new(ctx),
        }
    }
}

impl<T: Window> mq::EventHandler for InternalWindow<T> {
    fn update(&mut self, ctx: &mut mq::Context) {
        if !self.running.load(Ordering::Relaxed) {
            ctx.quit();
            return;
        }

        self.window.write().update(ctx);
    }

    fn draw(&mut self, ctx: &mut mq::Context) {
        // Unwrapping is fine since should never draw by itself.
        self.window.write().draw(ctx, &mut self.egui_ctx);
    }

    fn quit_requested_event(&mut self, _ctx: &mut mq::Context) {
        self.running.store(false, Ordering::Relaxed);
    }

    fn mouse_motion_event(&mut self, ctx: &mut mq::Context, x: f32, y: f32) {
        if self
            .window
            .write()
            .on_event(ctx, Event::MouseMotion { x, y })
        {
            self.egui_ctx.mouse_motion_event(x, y);
        }
    }

    fn mouse_wheel_event(&mut self, ctx: &mut mq::Context, dx: f32, dy: f32) {
        if self
            .window
            .write()
            .on_event(ctx, Event::MouseWheel { dx, dy })
        {
            self.egui_ctx.mouse_wheel_event(dx, dy);
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        if self
            .window
            .write()
            .on_event(ctx, Event::MouseDown { mb, x, y })
        {
            self.egui_ctx.mouse_button_down_event(ctx, mb, x, y);
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        if self
            .window
            .write()
            .on_event(ctx, Event::MouseUp { mb, x, y })
        {
            self.egui_ctx.mouse_button_up_event(ctx, mb, x, y);
        }
    }

    fn char_event(
        &mut self,
        ctx: &mut mq::Context,
        character: char,
        keymods: mq::KeyMods,
        repeat: bool,
    ) {
        if self.window.write().on_event(
            ctx,
            Event::Char {
                character,
                keymods,
                repeat,
            },
        ) {
            self.egui_ctx.char_event(character);
        }
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        repeat: bool,
    ) {
        if self.window.write().on_event(
            ctx,
            Event::KeyDown {
                keycode,
                keymods,
                repeat,
            },
        ) {
            self.egui_ctx.key_down_event(ctx, keycode, keymods);
        }
    }

    fn key_up_event(&mut self, ctx: &mut mq::Context, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        if self
            .window
            .write()
            .on_event(ctx, Event::KeyUp { keycode, keymods })
        {
            self.egui_ctx.key_up_event(keycode, keymods);
        }
    }
}

pub enum Event {
    KeyUp {
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
    },
    KeyDown {
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        repeat: bool,
    },
    Char {
        character: char,
        keymods: mq::KeyMods,
        repeat: bool,
    },
    MouseUp {
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    },
    MouseDown {
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    },
    MouseWheel {
        dx: f32,
        dy: f32,
    },
    MouseMotion {
        x: f32,
        y: f32,
    },
}

pub trait Window: Send + Sync {
    fn config(&self) -> mq::conf::Conf;

    /// Called when the window is opened.
    /// A new graphics context is initialized so you should initialize/re-initialize your resources here.
    fn on_open(&mut self, _ctx: &mut mq::Context) {}

    fn update(&mut self, ctx: &mut mq::Context);
    fn draw(&mut self, ctx: &mut mq::Context, egui_ctx: &mut egui_mq::EguiMq);

    /// Handle a window event.
    /// This should return false if the event shouldn't be passed to egui.
    fn on_event(&mut self, _ctx: &mut mq::Context, _event: Event) -> bool {
        true
    }
}

/// A window which can be started on it's own thread.
/// The window state is persisted across opens/closes.
pub struct WindowContainer<T: Window> {
    window: Arc<RwLock<T>>,
    running: Arc<AtomicBool>,
}

impl<T: Window> Deref for WindowContainer<T> {
    type Target = Arc<RwLock<T>>;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl<T: Window> WindowContainer<T> {
    pub fn new(window: T) -> Self {
        Self {
            window: Arc::new(RwLock::new(window)),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Close the window and destroy existing contexts.
    pub fn close(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Is the window currently open.
    pub fn is_open(&mut self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn open(&mut self) -> JoinHandle<()>
    where
        T: 'static,
    {
        if self.running.load(Ordering::Relaxed) {
            panic!("Already open!")
        } else {
            self.running.store(true, Ordering::Relaxed);

            let window_clone = self.window.clone();
            let running_clone = self.running.clone();
            thread::spawn(|| {
                let config = window_clone.read().config();
                mq::start(config, |ctx| {
                    Box::new(InternalWindow::new(ctx, running_clone, window_clone))
                });
            })
        }
    }
}

pub fn window_title(name: &str) -> String {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    format!("Chippyverse {} - {}", VERSION, name)
}
