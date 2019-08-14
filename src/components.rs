#[derive(Copy, Clone)]
pub struct Sprite {
    pub xy: (f32, f32),
    pub uv: (f32, f32),
    pub wh: (f32, f32),
}

#[derive(Copy, Clone)]
pub struct Physics {
    pub pos: (f32, f32),
    pub vel: (f32, f32),
    pub acc: (f32, f32),
}

pub struct SyncSpriteToPhysics;
