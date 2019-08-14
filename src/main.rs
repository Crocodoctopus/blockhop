#[macro_use]
extern crate lazy_static;
extern crate crossbeam_channel;
extern crate ezgl;
extern crate gl;
extern crate glutin;
extern crate nalgebra;
extern crate nphysics2d;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate compy;
extern crate rand;

mod camera;
mod components;
mod io;
mod render;
mod time;
mod update;

use glutin::{dpi::LogicalSize, ContextBuilder, EventsLoop, GlWindow, WindowBuilder};

fn main() {
    // window, loop and context
    let mut events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("Block Hop")
        .with_dimensions(LogicalSize::new(1280., 720.));
    let context = ContextBuilder::new().with_vsync(true);
    let window = GlWindow::new(window_builder, context, &events_loop).unwrap();

    // render state send/recv pa`
    let (render_send, render_recv) = crossbeam_channel::bounded(0);

    // input send/recv pair
    let (input_send, input_recv) = crossbeam_channel::unbounded();

    // update thread
    let update = std::thread::spawn(move || crate::update::update(render_send, input_recv));

    // draw thread
    let render = std::thread::spawn(move || crate::render::render(window, render_recv));

    // input "thread"
    events_loop.run_forever(|event| match input_send.send(event) {
        Ok(_) => glutin::ControlFlow::Continue,
        Err(_) => glutin::ControlFlow::Break,
    });

    // wait
    update.join().unwrap().unwrap();
    render.join().unwrap().unwrap();
}
