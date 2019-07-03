use crate::render::RenderState;
use crossbeam_channel::{Receiver, Sender};
use glutin::{
    Event::{self, WindowEvent},
    WindowEvent::*,
};

macro_rules! fixed_loop {
    ($sim_time: ident, $dt: ident, $u: expr, $d: expr) => {{
        // timing stuff
        let mut last_update = crate::time::get_microseconds_as_u64();

        // the game loop
        loop {
            // calculate how much time needs to be simulated
            let mut acc = crate::time::get_microseconds_as_u64() - last_update;

            // simulate it
            while acc > 0 {
                // calculate the lesser of either the remaining time, or the timestep cap
                let time_to_simulate = std::cmp::min(acc, $sim_time);
                acc -= time_to_simulate; // reduce remaining time

                // run the update step
                $dt = time_to_simulate;
                $u
            }

            // set last update to the end of the frame
            last_update = crate::time::get_microseconds_as_u64();

            // run the draw prepare step
            $d
        }
    }};
}

#[derive(Debug)]
pub enum UpdateErr {
    Location(u32, u32),
}

pub fn update(
    render_send: Sender<RenderState>,
    input_recv: Receiver<Event>,
) -> Result<(), UpdateErr> {
    // initialization state

    let sim_time = 66_666;
    let mut dt = 0u64;
    fixed_loop!(
        sim_time,
        dt,
        {
            // event poll
            for event in input_recv.try_iter() {
                match event {
                    WindowEvent {
                        event: CloseRequested,
                        ..
                    } => return Ok(()),
                    _ => {}
                }
            }

            // update
            // ??
        },
        {
            let render_state = RenderState {};
            render_send
                .send(render_state)
                .map_err(|_| UpdateErr::Location(column!(), line!()))?;
        }
    )
}
