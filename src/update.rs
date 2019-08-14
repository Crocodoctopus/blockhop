use crate::{
    components::{Physics, Sprite, SyncSpriteToPhysics},
    render::RenderState,
    time::get_microseconds_as_u64,
};
use compy::{compy::*, compy_builder::CompyBuilder, key::Key};
use crossbeam_channel::{Receiver, Sender};
use glutin::{
    Event::{self, WindowEvent},
    WindowEvent::*,
};
use nalgebra::{Isometry2, Point2, Vector2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::{
    force_generator::DefaultForceGeneratorSet,
    joint::DefaultJointConstraintSet,
    math::{Inertia, Velocity},
    object::{
        BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodySet, DefaultColliderSet, RigidBodyDesc,
    },
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};
use rand::prelude::*;

#[derive(Debug)]
pub enum UpdateErr {
    Location(u32, u32),
}

pub fn update(
    render_send: Sender<RenderState>,
    input_recv: Receiver<Event>,
) -> Result<(), UpdateErr> {
    // world
    let mut mechanical_world = DefaultMechanicalWorld::new(Vector2::new(0., 9.8));
    let mut geometrical_world = DefaultGeometricalWorld::new();
    let mut bodies = DefaultBodySet::new();
    let mut colliders = DefaultColliderSet::new();
    let mut joint_constraints = DefaultJointConstraintSet::new();
    let mut force_generators = DefaultForceGeneratorSet::new();

    // the ecs
    let mut compy = CompyBuilder::new()
        .with::<Sprite>()
        .with::<Physics>()
        .with::<SyncSpriteToPhysics>()
        .build();

    let none_key = Key::default();
    let sprite_key = compy.get_key_for::<Sprite>();
    let physics_key = compy.get_key_for::<Physics>();
    let sync_sprite_to_physics_key = compy.get_key_for::<SyncSpriteToPhysics>();

    // TEST INSERT ENTITY
    let rigid_body = RigidBodyDesc::new()
        .translation(Vector2::new(0., 0.))
        .mass(5.)
        .build();
    let rigid_body_handle = bodies.insert(rigid_body);
    let collider = ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector2::new(8., 8.))))
        .translation(Vector2::new(0., 0.))
        .build(BodyPartHandle(rigid_body_handle, 0));
    let collider_handle = colliders.insert(collider);
    compy.insert((
        Sprite {
            xy: (99., 99.),
            uv: (0., 0.),
            wh: (16., 16.),
        },
        Physics {
            body: rigid_body_handle,
            col: collider_handle,
        },
        SyncSpriteToPhysics,
    ));

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

            // perform some quick physics
            /*let pkey = physics_key;
            let nkey = none_key;
            compy.iterate_mut(pkey, nkey, |phys: &mut Physics| {
                phys.pos.0 += 0.5 * phys.acc.0 * dt * dt + phys.vel.0 * dt;
                phys.pos.1 += 0.5 * phys.acc.1 * dt * dt + phys.vel.1 * dt;
                phys.vel.0 += phys.acc.0 * dt;
                phys.vel.1 += phys.acc.1 * dt;
                phys.pos.1 > 720.
            });*/

            // map the sprites to the physics
            let pkey = sprite_key + physics_key + sync_sprite_to_physics_key;
            let nkey = none_key;
            compy.iterate_mut(pkey, nkey, |spr: &mut Sprite, phys: &Physics| {
                let t = colliders.get(phys.col).unwrap();
                let pos = t.position().translation.vector;
                println!("{:?}", pos);
                spr.xy = (pos.x, pos.y);
                false
            });

            // update nphysics2d
            mechanical_world.set_timestep(dt);
            mechanical_world.step(
                &mut geometrical_world,
                &mut bodies,
                &mut colliders,
                &mut joint_constraints,
                &mut force_generators,
            );

            // update ecs
            compy.update();
        }

        // prepare the render state and pass it to the gpu
        // (this only happens after all time for a frame is simulated (see above))
        let mut sprites = Vec::with_capacity(10000);
        let pkey = sprite_key;
        let nkey = none_key;
        compy.iterate_mut(pkey, nkey, |spr: &Sprite| {
            sprites.push(*spr);
            false
        });

        let render_state = RenderState {
            sprites,
            debug: true,
        };
        render_send
            .send(render_state)
            .map_err(|_| UpdateErr::Location(column!(), line!()))?;
    }
}
