pub struct Sprite {
	pub world_rect: Bound,
	pub texture_rect: Bound,
}

pub struct Animation {
	pub offset: f32,
	pub interval: f32,
	pub max_interval: f32,
}

pub struct Hitbox {
	pub bounds: SensorHandle,
}

pub struct Physics {
	pub handle: BodyHandle,
}