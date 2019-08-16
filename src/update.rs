use crate::{components::*, render::RenderState, time::get_microseconds_as_u64};
use compy::{compy::*, compy_builder::CompyBuilder, key::Key};
use crossbeam_channel::{Receiver, Sender};
use glutin::{
    ElementState,
    Event::{self, WindowEvent},
    MouseButton,
    WindowEvent::*,
};
use nalgebra::Vector2;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::{
    force_generator::DefaultForceGeneratorSet,
    joint::DefaultJointConstraintSet,
    math::Isometry,
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
        .with::<SpriteXY>()
        .with::<SpriteUV>()
        .with::<SpriteWH>()
        .with::<SpriteR>()
        .with::<PhysicsBody>()
        .with::<PhysicsCollider>()
        .with::<SyncSpriteToPhysics>()
        .with::<CursorSnapSpriteToGrid>()
        .with::<SetUVOnClickDown>()
        .with::<SetUVOnClickUp>()
        .build();
    let none_key = Key::default();
    let sprite_xy_key = compy.get_key_for::<SpriteXY>();
    let sprite_uv_key = compy.get_key_for::<SpriteUV>();
    let sprite_wh_key = compy.get_key_for::<SpriteWH>();
    let sprite_r_key = compy.get_key_for::<SpriteR>();
    let physics_body_key = compy.get_key_for::<PhysicsBody>();
    let physics_collider_key = compy.get_key_for::<PhysicsCollider>();
    let sync_sprite_to_physics_key = compy.get_key_for::<SyncSpriteToPhysics>();
    let cursor_snap_sprite_to_grid_key = compy.get_key_for::<CursorSnapSpriteToGrid>();
    let set_uv_on_click_up_key = compy.get_key_for::<SetUVOnClickUp>();
    let set_uv_on_click_down_key = compy.get_key_for::<SetUVOnClickDown>();

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
    crate::components::create_sprite((0., camh - 80.), (352., 0.), (352., 80.), &compy);
    crate::components::create_wall(
        (0., camh - 32. - 48.),
        (64., 48.),
        &compy,
        world,
        &mut colliders,
    );
    // wall seg1
    let wall_size = 48.;
    crate::components::create_sprite((0., camh - 80. - 48.), (0., 0.), (352., 48.), &compy);
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
    crate::components::create_sprite((0., camh - 80. - 48. - 48.), (0., 0.), (352., 48.), &compy);
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
    crate::components::create_cursor(&compy);
    crate::components::create_normal_block_particles((128., 0.), &compy, &mut bodies);

    // extra data
    let mut compy_stat_counter = 0f32;
    let mut block_drop_counter = 0f32;
    let mut cursor_x = 0.;
    let mut cursor_y = 0.;
    let mut cursor_left_down = false;
    let mut cursor_last_left_down = false;

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
            let mut click_pressed = false;
            let mut click_released = false;
            for event in input_recv.try_iter() {
                match event {
                    WindowEvent {
                        event: CloseRequested,
                        ..
                    } => return Ok(()),
                    WindowEvent {
                        event: CursorMoved { position, .. },
                        ..
                    } => {
                        cursor_x = position.x as f32;
                        cursor_y = position.y as f32;
                    }
                    WindowEvent {
                        event: MouseInput { state, button, .. },
                        ..
                    } => match (state, button) {
                        (ElementState::Pressed, MouseButton::Left) => click_pressed = true,
                        (ElementState::Released, MouseButton::Left) => click_released = true,
                        _ => {}
                    },
                    _ => {}
                }
            }

            // randomly spawn a normal block every 3 seconds
            block_drop_counter += dt;
            if block_drop_counter > 3f32 {
                block_drop_counter -= 3f32;

                let x = 64 + 16 + (rand::random::<u32>() % 7) * 32;
                crate::components::create_normal_block(
                    (x as f32, -16.),
                    &compy,
                    &mut bodies,
                    &mut colliders,
                );

                compy.update();
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

            // if click was recently pressed, update on click systems
            if click_pressed {
                let pkey = set_uv_on_click_down_key + sprite_uv_key;
                compy.iterate_mut(
                    pkey,
                    none_key,
                    |click_down_uv: &SetUVOnClickDown, sprite_uv: &mut SpriteUV| {
                        sprite_uv.0 = click_down_uv.0;
                        sprite_uv.1 = click_down_uv.1;
                        false
                    },
                );
            }

            // if click was recently released, update on click release systems
            if click_released {
                let pkey = set_uv_on_click_up_key + sprite_uv_key;
                compy.iterate_mut(
                    pkey,
                    none_key,
                    |click_up_uv: &SetUVOnClickUp, sprite_uv: &mut SpriteUV| {
                        sprite_uv.0 = click_up_uv.0;
                        sprite_uv.1 = click_up_uv.1;
                        false
                    },
                );
            }

            // map the sprite xy to the cursor position
            let pkey = cursor_snap_sprite_to_grid_key + sprite_xy_key;
            // calculate
            let norm = 80.;
            let temp_x = ((cursor_x / 3. - norm) / 32.).round() * 32. + norm;
            let temp_x = nalgebra::clamp(temp_x, norm, 272.);
            let norm = camh - 32. - 16.;
            let temp_y = ((cursor_y / 3. - norm) / 32.).round() * 32. + norm;
            let temp_y = nalgebra::clamp(temp_y, -9999999., norm);
            let cursor_isom = Isometry::new(Vector2::new(temp_x, temp_y), 0.);
            compy.iterate_mut(pkey, none_key, |sprite_xy: &mut SpriteXY| {
                sprite_xy.0 = temp_x - 16.;
                sprite_xy.1 = temp_y - 16.;
                false
            });

            // update ecs
            compy.update();

            // print stats
            compy_stat_counter += dt;
            if compy_stat_counter > 10. {
                compy_stat_counter -= 10.;
                compy.print_stats();
            }
        }

        ///////////////////////////////////////////
        // prepare the render state and pass it to the gpu
        // (this only happens after all time for a frame is simulated (see above))

        // map the sprites position to the physics position
        let pkey = sprite_xy_key + sprite_r_key + physics_body_key + sync_sprite_to_physics_key;
        compy.iterate_mut(
            pkey,
            none_key,
            |sprite_xy: &mut SpriteXY, sprite_r: &mut SpriteR, phys: &PhysicsBody| {
                let pos = bodies.rigid_body(phys.0).unwrap().position();
                sprite_xy.0 = pos.translation.vector.x;
                sprite_xy.1 = pos.translation.vector.y;
                let rot = pos.rotation.into_inner();
                sprite_r.0 = rot.im.atan2(rot.re);
                false
            },
        );

        // pull some data out of the ECS for the renderer
        let mut sprite_xys = Vec::new();
        compy.iterate_mut(sprite_xy_key, none_key, |sprite_xy: &SpriteXY| {
            sprite_xys.push((sprite_xy.0, sprite_xy.1));
            false
        });

        let mut sprite_uvs = Vec::new();
        compy.iterate_mut(sprite_uv_key, none_key, |sprite_uv: &SpriteUV| {
            sprite_uvs.push((sprite_uv.0, sprite_uv.1));
            false
        });

        let mut sprite_whs = Vec::new();
        compy.iterate_mut(sprite_wh_key, none_key, |sprite_wh: &SpriteWH| {
            sprite_whs.push((sprite_wh.0, sprite_wh.1));
            false
        });

        let mut sprite_rghs = Vec::new();
        compy.iterate_mut(sprite_r_key, none_key, |sprite_r: &SpriteR| {
            sprite_rghs.push((sprite_r.0, sprite_r.1, sprite_r.2));
            false
        });

        // generate wirebox data for the renderer
        let mut wireboxes = Vec::new();
        compy.iterate_mut(physics_collider_key, none_key, |phys: &PhysicsCollider| {
            let t = colliders.get(phys.0).unwrap();
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

        // generate body data for the renderer
        let mut rigid_bodies = Vec::new();
        compy.iterate_mut(physics_body_key, none_key, |physics_body: &PhysicsBody| {
            let pos = bodies
                .rigid_body(physics_body.0)
                .unwrap()
                .position()
                .translation
                .vector;
            rigid_bodies.push((pos.x, pos.y));
            false
        });

        let render_state = RenderState {
            sprite_xys,
            sprite_uvs,
            sprite_whs,
            sprite_rghs,
            debug: false,
            wireboxes: Some(wireboxes),
            rigid_bodies: Some(rigid_bodies),
        };
        render_send
            .send(render_state)
            .map_err(|_| UpdateErr::Location(column!(), line!()))?;
    }
}
