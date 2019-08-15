use compy::compy::Compy;
use compy::{compy::*};
use nalgebra::Vector2;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::object::{
    BodyPartHandle, ColliderDesc, DefaultBodyHandle, DefaultColliderHandle,
    DefaultColliderSet,
};

#[derive(Copy, Clone)]
pub struct Sprite {
    pub xy: (f32, f32),
    pub uv: (f32, f32),
    pub wh: (f32, f32),
}

#[derive(Copy, Clone)]
pub struct Physics {
    pub body: DefaultBodyHandle,
    pub col: DefaultColliderHandle,
}

pub struct SyncSpriteToPhysics;

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
    world: DefaultColliderHandle,
    colliders: &mut DefaultColliderSet<f32>,
) {
    let collider = ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector2::new(wh.0 /2., wh.1 / 2.))))
        .translation(Vector2::new(xy.0 + wh.0 / 2., xy.1 + wh.1 / 2.))
        .build(BodyPartHandle(world, 0));
    let collider_handle = colliders.insert(collider);
    compy.insert((Physics {
        body: world,
        col: collider_handle,
    },));
}
