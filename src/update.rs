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
    math::{Isometry, Point},
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
    let mut mechanical_world = DefaultMechanicalWorld::new(Vector2::new(0., 19.8));
    let mut geometrical_world = DefaultGeometricalWorld::<f32>::new();
    let mut bodies = DefaultBodySet::new();
    let mut colliders = DefaultColliderSet::new();
    let mut joint_constraints = DefaultJointConstraintSet::<f32>::new();
    let mut force_generators = DefaultForceGeneratorSet::<f32>::new();

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
        .with::<CursorEmitDestroyEventOnLMBDown>()
        .with::<SetUVOnLMBDown>()
        .with::<SetUVOnLMBUp>()
        .with::<HP>()
        .with::<TakeCursorDamage>()
        .with::<KillUpon0HP>()
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
    let cursor_emit_destroy_event_on_lmb_down_key =
        compy.get_key_for::<CursorEmitDestroyEventOnLMBDown>();
    let set_uv_on_lmb_up_key = compy.get_key_for::<SetUVOnLMBUp>();
    let set_uv_on_lmb_down_key = compy.get_key_for::<SetUVOnLMBDown>();
    let hp_key = compy.get_key_for::<HP>();
    let take_cursor_damage_key = compy.get_key_for::<TakeCursorDamage>();
    let kill_upon_0_hp_key = compy.get_key_for::<KillUpon0HP>();

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
    //crate::components::create_normal_block_particles((128., 0.), &compy, &mut bodies);

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
            let ft_start = get_microseconds_as_u64();

            // event poll
            let mut lmb_pressed = false;
            let mut lmb_released = false;
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
                        (ElementState::Pressed, MouseButton::Left) => lmb_pressed = true,
                        (ElementState::Released, MouseButton::Left) => lmb_released = true,
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

            // if lmb was recently pressed, update on lmb systems
            if lmb_pressed {
                // cursor "on press" event
                let pkey = set_uv_on_lmb_down_key + sprite_uv_key;
                compy.iterate_mut(
                    pkey,
                    none_key,
                    |lmb_down_uv: &SetUVOnLMBDown, sprite_uv: &mut SpriteUV| {
                        sprite_uv.0 = lmb_down_uv.0;
                        sprite_uv.1 = lmb_down_uv.1;
                        false
                    },
                );

                // generate lmb events
                let mut lmb_events = Vec::new();
                let pkey = cursor_emit_destroy_event_on_lmb_down_key;
                compy.iterate_mut(pkey, none_key, || {
                    lmb_events.push(Point::new(cursor_x/3., cursor_y/3.));
                    false
                });

                // handle lmb events
                if lmb_events.len() > 0 {
                    let pkey = take_cursor_damage_key + hp_key + physics_collider_key;
                    compy.iterate_mut(
                        pkey,
                        none_key,
                        |hp: &mut HP, physics_collider: &PhysicsCollider| {
                            let collider = colliders.get(physics_collider.0).unwrap();
                            let iso = collider.position();
                            let shape = collider.shape().downcast_ref::<Cuboid<f32>>().unwrap();

                            for p in &lmb_events {
                                use crate::ncollide2d::query::PointQuery;
                                if shape.contains_point(&iso, &p) {
                                    hp.0 -= 1;
                                }
                            }

                            false
                        },
                    );
                }
            }

            // if lmb was recently released, update on lmb release systems
            if lmb_released {
                let pkey = set_uv_on_lmb_up_key + sprite_uv_key;
                compy.iterate_mut(
                    pkey,
                    none_key,
                    |lmb_up_uv: &SetUVOnLMBUp, sprite_uv: &mut SpriteUV| {
                        sprite_uv.0 = lmb_up_uv.0;
                        sprite_uv.1 = lmb_up_uv.1;
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

            // destroy entities with <0 HP
            let pkey = hp_key + kill_upon_0_hp_key;
            compy.iterate_mut(pkey, none_key, |hp: &HP| {
                hp.0 == 0
            });

            // update ecs
            compy.update();

            // print stats
            compy_stat_counter += dt;
            if compy_stat_counter > 10. {
                compy_stat_counter -= 10.;
                println!("ft: {:?}", get_microseconds_as_u64() - ft_start);
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
