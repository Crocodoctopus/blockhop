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
#[derive(Copy, Clone)]
pub struct SpriteR(pub f32, pub f32, pub f32); // r, cx, cy

// Some physics stuff
#[derive(Copy, Clone)]
pub struct PhysicsBody(pub DefaultBodyHandle);
#[derive(Copy, Clone)]
pub struct PhysicsCollider(pub DefaultColliderHandle);

// Sync the sprite to the physics body just before render
#[derive(Copy, Clone)]
pub struct SyncSpriteToPhysics;

// HP and damage tags
#[derive(Copy, Clone)]
pub struct HP(pub u8); // no max
#[derive(Copy, Clone)]
pub struct TakeCursorDamage; // use PhysicsCollider(maybe a new collider?)

// kill on tags
/*#[derive(Copy, Clone)]
pub struct KillUponLeavingScreen; // Uses PhysicsBody for position*/
#[derive(Copy, Clone)]
pub struct KillUpon0HP;

// UV setting, based on mouse holding state, pretty straightforward
#[derive(Copy, Clone)]
pub struct SetUVOnLMBUp(pub f32, pub f32);
#[derive(Copy, Clone)]
pub struct SetUVOnLMBDown(pub f32, pub f32);

// Some special cursor states
#[derive(Copy, Clone)]
pub struct CursorSnapSpriteToGrid; // snaps to the play area grid
#[derive(Copy, Clone)]
pub struct CursorEmitDestroyEventOnLMBDown; // emits a "destroy event" at the cursor location

// creates an unmoving, uncollidable, sprite with with xy being the top left corner
pub fn create_sprite(xy: (f32, f32), uv: (f32, f32), wh: (f32, f32), compy: &Compy) {
    compy.insert((
        SpriteXY(xy.0, xy.1),
        SpriteUV(uv.0, uv.1),
        SpriteWH(wh.0, wh.1),
        SpriteR(0., 0., 0.),
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
        .velocity(Velocity::linear(0.0, 32.0))
        //.max_linear_velocity(64.0)
        .mass(0.02)
        .build();
    let rigid_body_handle = bodies.insert(rigid_body);
    let collider = ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector2::new(15., 16.))))
        .translation(Vector2::new(0., 0.))
        .build(BodyPartHandle(rigid_body_handle, 0));
    let collider_handle = colliders.insert(collider);
    compy.insert((
        SpriteXY(xy.0, xy.1),
        SpriteUV(352., 144.),
        SpriteWH(32., 32.),
        SpriteR(0., -16., -16.),
        PhysicsBody(rigid_body_handle),
        PhysicsCollider(collider_handle),
        SyncSpriteToPhysics,
        HP(1),
        TakeCursorDamage,
        KillUpon0HP,
    ));
}

pub fn create_normal_block_particles(
    xy: (f32, f32),
    compy: &Compy,
    bodies: &mut DefaultBodySet<f32>,
) {
    let rigid_body = RigidBodyDesc::new()
        .translation(Vector2::new(xy.0, xy.1))
        .velocity(Velocity::new(Vector2::new(-8., 16.), -2.))
        .build();
    let rigid_body_handle = bodies.insert(rigid_body);
    compy.insert((
        SpriteXY(xy.0, xy.1),
        SpriteUV(352., 144.),
        SpriteWH(16., 16.),
        SpriteR(0., -8., -8.),
        PhysicsBody(rigid_body_handle),
        SyncSpriteToPhysics,
    ));
}

pub fn create_cursor(compy: &Compy) {
    compy.insert((
        SpriteXY(-99999., -99999.),
        SpriteUV(576., 208.),
        SpriteWH(32., 32.),
        SpriteR(0., 0., 0.),
        CursorSnapSpriteToGrid,
        SetUVOnLMBUp(576., 208.),
        SetUVOnLMBDown(576. + 32., 208.),
        CursorEmitDestroyEventOnLMBDown,
    ));
}
