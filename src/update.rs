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
use nalgebra::Vector2;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::{
    force_generator::DefaultForceGeneratorSet,
    joint::DefaultJointConstraintSet,
    object::{
        BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodySet, DefaultColliderSet, RigidBodyDesc,
    },
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};

#[derive(Debug)]
pub enum UpdateErr {
    Location(u32, u32),
}

pub fn update(
    camw: f32,
    camh: f32,
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

    // the world is a special permanent handle that is unmoving
    let world = RigidBodyDesc::new().status(BodyStatus::Static).build();
    let world = bodies.insert(world);
    // bottom
    crate::components::create_wall(
        (64., camh - 32.),
        (288., 32.),
        &compy,
        world,
        &mut colliders,
    );
    crate::components::create_sprite((0., camh - 80.), (352., 48.), (352., 80.), &compy);
    crate::components::create_wall(
        (0., camh - 32. - 48.),
        (64., 48.),
        &compy,
        world,
        &mut colliders,
    );
    // wall seg1
    let wall_size = 48.;
    crate::components::create_sprite((0., camh - 80. - 48.), (0., 48.), (352., 48.), &compy);
    crate::components::create_wall(
        (0., camh - 32. - 48. - 48.),
        (64., wall_size),
        &compy,
        world,
        &mut colliders,
    );
    crate::components::create_wall(
        (288., camh - 32. - 48. - 48.),
        (64., wall_size),
        &compy,
        world,
        &mut colliders,
    );
    // wall seg1
    let wall_size = 48.;
    crate::components::create_sprite((0., camh - 80. - 48. - 48.), (0., 48.), (352., 48.), &compy);
    crate::components::create_wall(
        (0., camh - 32. - 48. - 48. - 48.),
        (64., wall_size),
        &compy,
        world,
        &mut colliders,
    );
    crate::components::create_wall(
        (288., camh - 32. - 48. - 48. - 48.),
        (64., wall_size),
        &compy,
        world,
        &mut colliders,
    );
    // temp
    crate::components::create_normal_block((64. + 16., 0.), &compy, &mut bodies, &mut colliders);

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
        let mut draw = false;
        while acc > 0 {
            draw = true;
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

            // update nphysics2d
            mechanical_world.set_timestep(dt);
            mechanical_world.step(
                &mut geometrical_world,
                &mut bodies,
                &mut colliders,
                &mut joint_constraints,
                &mut force_generators,
            );

            // map the sprites to the physics
            let pkey = sprite_key + physics_key + sync_sprite_to_physics_key;
            let nkey = none_key;
            compy.iterate_mut(pkey, nkey, |spr: &mut Sprite, phys: &Physics| {
                let t = colliders.get(phys.col).unwrap();
                let pos = t.position().translation.vector;
                spr.xy = (pos.x - spr.wh.0 / 2., pos.y - spr.wh.1 / 2.);
                false
            });

            // update ecs
            compy.update();
        }

        // prepare the render state and pass it to the gpu
        // (this only happens after all time for a frame is simulated (see above))
        //std::thread::sleep(std::time::Duration::from_millis(1000));
        let mut sprites = Vec::new();
        compy.iterate_mut(sprite_key, none_key, |spr: &Sprite| {
            sprites.push(*spr);
            false
        });

        let mut wireboxes = Vec::new();
        compy.iterate_mut(physics_key, none_key, |phys: &Physics| {
            let t = colliders.get(phys.col).unwrap();
            let xy = t.position().translation.vector;
            let wh_half = t
                .shape()
                .downcast_ref::<Cuboid<f32>>()
                .unwrap()
                .half_extents();
            wireboxes.push((
                xy.x - wh_half.x,
                xy.y - wh_half.y,
                wh_half.x * 2.,
                wh_half.y * 2.,
            ));
            false
        });

        let render_state = RenderState {
            sprites,
            debug: true,
            wireboxes: Some(wireboxes),
        };
        render_send
            .send(render_state)
            .map_err(|_| UpdateErr::Location(column!(), line!()))?;
    }
}
