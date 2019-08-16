use compy::compy::*;
use nalgebra::Vector2;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::{
    math::Velocity,
    object::{
        BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodyHandle, DefaultBodySet,
        DefaultColliderHandle, DefaultColliderSet, RigidBodyDesc,
    },
};

// A sprite
#[derive(Copy, Clone)]
pub struct Sprite {
    pub xy: (f32, f32),
    pub uv: (f32, f32),
    pub wh: (f32, f32),
}

// A body handle and collider
#[derive(Copy, Clone)]
pub struct Physics {
    pub body: DefaultBodyHandle,
    pub col: DefaultColliderHandle,
}

// [TODO: Write desc]
pub struct SyncSpriteToPhysics;

// [TODO: Write desc]
pub struct SyncPhysicsToCursor;

// prefabs

// creates an unmoving, uncollidable, sprite with with xy being the top left corner
pub fn create_sprite(xy: (f32, f32), uv: (f32, f32), wh: (f32, f32), compy: &Compy) {
    compy.insert((Sprite { xy, uv, wh },));
}

// creates an unmoving, solid, region with with xy being the top left corner
pub fn create_wall(
    xy: (f32, f32),
    wh: (f32, f32),
    compy: &Compy,
    world: DefaultBodyHandle,
    colliders: &mut DefaultColliderSet<f32>,
) {
    let collider = ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector2::new(
        wh.0 / 2.,
        wh.1 / 2.,
    ))))
    .translation(Vector2::new(xy.0 + wh.0 / 2., xy.1 + wh.1 / 2.))
    .build(BodyPartHandle(world, 0));
    let collider_handle = colliders.insert(collider);
    compy.insert((Physics {
        body: world,
        col: collider_handle,
    },));
}

pub fn create_normal_block(
    xy: (f32, f32),
    compy: &Compy,
    bodies: &mut DefaultBodySet<f32>,
    colliders: &mut DefaultColliderSet<f32>,
) {
    let rigid_body = RigidBodyDesc::new()
        .translation(Vector2::new(xy.0, xy.1))
        .mass(9999999.)
        .velocity(Velocity::linear(0.0, 32.0))
        .build();
    let rigid_body_handle = bodies.insert(rigid_body);
    let collider = ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector2::new(15., 16.))))
        .translation(Vector2::new(0., 0.))
        .build(BodyPartHandle(rigid_body_handle, 0));
    let collider_handle = colliders.insert(collider);
    compy.insert((
        Sprite {
            xy,
            uv: (352., 192.),
            wh: (32., 32.),
        },
        Physics {
            body: rigid_body_handle,
            col: collider_handle,
        },
        SyncSpriteToPhysics,
    ));
}

pub fn create_cursor(
    compy: &Compy,
    world: DefaultBodyHandle,
    colliders: &mut DefaultColliderSet<f32>,
) {
    let collider = ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector2::new(16., 16.))))
        .sensor(true)
        .build(BodyPartHandle(world, 0));
    let collider = colliders.insert(collider);
    compy.insert((
        Sprite {
            xy: (0., 0.),
            uv: (672., 160.),
            wh: (32., 32.),
        },
        Physics {
            body: world,
            col: collider,
        },
        SyncPhysicsToCursor,
        SyncSpriteToPhysics,
    ));
}
