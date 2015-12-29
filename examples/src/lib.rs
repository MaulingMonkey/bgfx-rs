// Copyright (c) 2015, Johan Sköld.
// License: http://opensource.org/licenses/ISC

extern crate bgfx;
extern crate glfw;
extern crate libc;

use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;

use glfw::{Context, Glfw, Window, WindowEvent};

/// Events received by the main thread, sent by the render thread.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Event {
    /// Window close event.
    Close,

    /// Window size event.
    Size(u16, u16),
}

/// Example application.
pub struct Example {
    /// Receiver for events from the render thread.
    event_rx: Receiver<Event>,
}

impl Example {

    /// Handles events received from the render thread. If there are no events to process, returns
    /// instantly.
    ///
    /// Returns `true` if the app should exit.
    pub fn handle_events(&self,
                         bgfx: &bgfx::MainContext,
                         width: &mut u16,
                         height: &mut u16,
                         reset: bgfx::ResetFlags)
                         -> bool {
        let mut close = false;

        loop {
            let result = self.event_rx.try_recv();

            if !result.is_ok() {
                break;
            }

            match result.ok().unwrap() {
                Event::Close => close = true,
                Event::Size(w, h) => {
                    *width = w;
                    *height = h;
                    bgfx.reset(w, h, reset);
                }
            }
        }


        close
    }

}

/// Example data used by the render thread.
struct ExampleData {
    /// The 'Glfw' object.
    glfw: Glfw,

    /// Receiver of window events from GLFW.
    events: Receiver<(f64, WindowEvent)>,

    /// The GLFW window object.
    window: Window,

    /// Sender of events to the main thread.
    event_tx: Sender<Event>,
}

impl ExampleData {

    /// Process GLFW events, and potentially forward them to the main thread.
    ///
    /// Returns `true` if the example should exit.
    fn process_events(&mut self) -> bool {
        self.glfw.poll_events();

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Close => {
                    self.event_tx.send(Event::Close).unwrap();
                }
                WindowEvent::Size(w, h) => {
                    self.event_tx.send(Event::Size(w as u16, h as u16)).unwrap();
                }
                ref e => {
                    panic!(format!("Unhandled event: {:?}", e))
                }
            }
        }

        self.window.should_close()
    }

}

/// Loads the contents of a file and returns it.
pub fn load_file(name: &str) -> Vec<u8> {
    let mut data = Vec::new();
    let mut file = File::open(name).unwrap();
    file.read_to_end(&mut data).unwrap();
    data
}

/// Loads the two given shaders from disk, and creates a program using the new
/// shaders.
///
/// # Arguments
///
/// * ``
pub fn load_program<'a, 'b>(bgfx: &'a bgfx::MainContext,
                            vsh_name: &'b str,
                            fsh_name: &'b str)
                            -> bgfx::Program<'a> {
    let renderer = bgfx.get_renderer_type();
    let vsh_path = format!("{:?}/{}.bin", renderer, vsh_name);
    let fsh_path = format!("{:?}/{}.bin", renderer, fsh_name);
    let vsh_mem = bgfx::Memory::copy(&load_file(&vsh_path));
    let fsh_mem = bgfx::Memory::copy(&load_file(&fsh_path));
    let vsh = bgfx::Shader::new(bgfx, vsh_mem);
    let fsh = bgfx::Shader::new(bgfx, fsh_mem);

    bgfx::Program::new(vsh, fsh)
}

/// Returns a new `bgfx::Application`.
///
/// # Arguments
///
/// * `glfw` - Reference to the `Glfw` object.
/// * `window` - Reference to the GLFW window object.
#[cfg(linux)]
fn create_bgfx(glfw: &Glfw, window: &Window) -> bgfx::Config {
    let mut bgfx = bgfx::create();
    bgfx.context(window.get_glx_context());
    bgfx.display(glfw.get_x11_display());
    bgfx.window(window.get_x11_window());
    bgfx
}

#[cfg(windows)]
fn create_bgfx(_: &Glfw, window: &Window) -> bgfx::Config {
    let mut bgfx = bgfx::create();
    bgfx.window(window.get_win32_window());
    bgfx
}

/// Determines the renderer to use based on platform.
///
/// The sole reason for using this instead of using `bgfx::RendererType::Default` is cause
/// `Direct3D12` - which is the default under Windows 10 - currently (2015-10-08) crashes when
/// compiled with MinGW. This is true with the examples in the C++ version of bgfx as well, and
/// not exlusive to Rust.
pub fn get_renderer_type() -> Option<bgfx::RendererType> {
    if cfg!(windows) && cfg!(target_env = "gnu") {
        Some(bgfx::RendererType::Direct3D11)
    } else {
        None
    }
}

/// Runs an example.
///
/// # Arguments
///
/// * `width` - Initial width of the window, in pixels.
/// * `height` - Initial height of the window, in pixels.
/// * `main` - Function to act as the entry point for the example.
pub fn run_example<M>(width: u32, height: u32, main: M)
    where M: Send + 'static + FnOnce(bgfx::MainContext, &Example)
{
    // Initialize GLFW and create the window.
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw.create_window(width,
                                                  height,
                                                  "BGFX",
                                                  glfw::WindowMode::Windowed)
                                   .expect("Failed to create GLFW window.");

    window.set_close_polling(true);
    window.set_size_polling(true);
    window.make_current();

    // Create the channel used for communication between the main and render threads.
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    // Initialize the example.
    let mut data = ExampleData {
        glfw: glfw,
        events: events,
        window: window,
        event_tx: event_tx,
    };

    let bgfx = create_bgfx(&data.glfw, &data.window);

    // Main thread implementation.
    let main_thread = move |bgfx: bgfx::MainContext| {
        let example = Example { event_rx: event_rx };
        main(bgfx, &example);
    };

    // Render thread implementation.
    let render_thread = |bgfx: bgfx::RenderContext| {
        while !data.process_events() {
            bgfx.render_frame();
        }
    };

    // Run the application
    bgfx.run(main_thread, render_thread).unwrap();
}