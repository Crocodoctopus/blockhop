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

// The properties of the final sprite to be rendered
#[derive(Copy, Clone)]
pub struct SpriteXY(pub f32, pub f32);
#[derive(Copy, Clone)]
pub struct SpriteUV(pub f32, pub f32);
#[derive(Copy, Clone)]
pub struct SpriteWH(pub f32, pub f32);

// Some physics stuff
#[derive(Copy, Clone)]
pub struct PhysicsBody(pub DefaultBodyHandle);
#[derive(Copy, Clone)]
pub struct PhysicsCollider(pub DefaultColliderHandle);

// Sync the sprite to the physics body just before render
// (this will use SpriteWH/2 and assume no rotation)
#[derive(Copy, Clone)]
pub struct SyncSpriteToPhysics;

// UV setting, based on mouse holding state, pretty straightforward
#[derive(Copy, Clone)]
pub struct SetUVOnClickUp(pub f32, pub f32);
#[derive(Copy, Clone)]
pub struct SetUVOnClickDown(pub f32, pub f32);

// Some special cursor states
#[derive(Copy, Clone)]
pub struct CursorSnapSpriteToGrid; // snaps to the play area grid
#[derive(Copy, Clone)]
pub struct CursorEmitDestroyEventOnClick; // emits a "destroy event" as the cursor location

// creates an unmoving, uncollidable, sprite with with xy being the top left corner
pub fn create_sprite(xy: (f32, f32), uv: (f32, f32), wh: (f32, f32), compy: &Compy) {
    compy.insert((
        SpriteXY(xy.0, xy.1),
        SpriteUV(uv.0, uv.1),
        SpriteWH(wh.0, wh.1),
    ));
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
    compy.insert((PhysicsCollider(collider_handle),));
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
        SpriteXY(xy.0, xy.1),
        SpriteUV(352., 192.),
        SpriteWH(32., 32.),
        PhysicsBody(rigid_body_handle),
        PhysicsCollider(collider_handle),
        SyncSpriteToPhysics,
    ));
}

pub fn create_cursor(compy: &Compy) {
    compy.insert((
        SpriteXY(-99999., -99999.),
        SpriteUV(672., 160.),
        SpriteWH(32., 32.),
        CursorSnapSpriteToGrid,
        SetUVOnClickUp(672., 160.),
        SetUVOnClickDown(704., 160.),
    ));
}
