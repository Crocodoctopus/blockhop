use nphysics2d::object::{DefaultBodyHandle, DefaultColliderHandle};

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
