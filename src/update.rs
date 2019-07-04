use crate::{
    etc::Bounds,
    handle::{Handle, HandleGenerator},
    render::RenderState,
    time::get_microseconds_as_u64,
};
use crossbeam_channel::{Receiver, Sender};
use glutin::{
    Event::{self, WindowEvent},
    WindowEvent::*,
};
use nalgebra::*;
use nphysics2d::{
    object::{BodyHandle, BodyStatus, ColliderDesc, RigidBodyDesc},
    world::World,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub enum UpdateErr {
    Location(u32, u32),
}

pub fn update(
    render_send: Sender<RenderState>,
    input_recv: Receiver<Event>,
) -> Result<(), UpdateErr> {
    // world
    let mut world = World::<f32>::new();
    world.set_gravity(Vector2::y() * 9.81); // is this needed?

    // id counter
    let mut handle_gen = HandleGenerator::new();

    // entity data
    let mut hitboxes = BTreeMap::<Handle, Bounds>::new(); // (xy)
    let mut sprites = BTreeMap::<Handle, (Bounds, Bounds)>::new(); // (xy, uv)
    let mut rbody_handles = BTreeMap::<Handle, BodyHandle>::new();
    let mut sync = BTreeSet::<Handle>::new(); // syncs hitbox <-> sprite <-> rbody

    // create player
    {
        let h = handle_gen.gen();
        hitboxes.insert(h, Bounds::new(0., 0., 32., 32.));
        sprites.insert(
            h,
            (Bounds::new(0., 0., 32., 32.), Bounds::new(0., 0., 32., 32.)),
        );
        rbody_handles.insert(
            h,
            RigidBodyDesc::new()
                .position(Isometry2::new(Vector2::new(32.0, 32.0), 3.14))
                .gravity_enabled(true)
                .mass(1.2)
                .build(&mut world)
                .handle(),
        );
        sync.insert(h);
    }

    // game loop
    //  The inner update loop will simulate the amount of time elapsed since the start
    //  of the next frame, in whole chunks no larger than sim_time. The frame is only
    //  pushed to the render task once all the time as been elapsed.
    let sim_time = 66_666; // 66_666us = 66.666ms = 0.066ms
    let mut last_update = get_microseconds_as_u64();
    loop {
        // calculate how much time needs to be simulated
        let now = get_microseconds_as_u64();
        let mut acc = now - last_update;
        last_update = now;

        // keep running the update process until all time has been simulated
        while acc > 0 {
            let time_to_simulate = std::cmp::min(acc, sim_time);
            acc -= time_to_simulate;

            ///////////////////////////////////////////
            // update
            let dt = time_to_simulate as f32 * 0.000001;

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

            // sync hitboxes to rbodies, then sprites to hitboxes
            {
                for sync_ids in &sync {
                    let rbody_handle = *rbody_handles.get(sync_ids).unwrap();
                    let mut hitbox = hitboxes.get_mut(sync_ids).unwrap();
                    let rbody = world.rigid_body(rbody_handle).unwrap();
                    hitbox.x = rbody.position().translation.vector.x;
                    hitbox.y = rbody.position().translation.vector.y;

                    let mut sprite = sprites.get_mut(sync_ids).unwrap();
                    sprite.0.x = hitbox.x;
                    sprite.0.y = hitbox.y;
                }
            }

            // update nphysics2d
            world.set_timestep(dt);
            world.step();
        }

        // prepare the render state and pass it to the gpu
        // (this only happens after all time for a frame is simulated (see above))
        let render_state = RenderState {
            hitboxes: hitboxes.clone(),
            sprites: sprites.clone(),
            debug: true,
        };
        render_send
            .send(render_state)
            .map_err(|_| UpdateErr::Location(column!(), line!()))?;
    }
}
