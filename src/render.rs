use crate::glutin::GlWindow;
use crossbeam_channel::Receiver;

pub struct RenderState {}

#[derive(Debug)]
pub enum RenderErr {
    Location(u32, u32),
}

pub fn render(window: GlWindow, render_recv: Receiver<RenderState>) -> Result<(), RenderErr> {
    // build gl context
    unsafe {
        use crate::glutin::GlContext;

        window
            .make_current()
            .map_err(|_| RenderErr::Location(column!(), line!()))?;
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0., 1., 0., 1.);
    }

    loop {
        // get a render state from the update thread
        let render_state = match render_recv.recv() {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        // clear
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // render

        // swap buffer
        window
            .swap_buffers()
            .map_err(|_| RenderErr::Location(column!(), line!()))?;
    }
}
