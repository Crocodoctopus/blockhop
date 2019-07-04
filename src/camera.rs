use ezgl::*;

pub fn camera(x: f32, y: f32, w: f32, h: f32) -> Mat4 {
    use nalgebra::*;

    // create ortho
    let mut matrix = Orthographic3::new(0., w, 0., h, -100., 100.).into_inner();

    // since ortho doesn't accept negative values, we have to cheat to set it up properly
    matrix *= Matrix4::new_nonuniform_scaling(&Vector3::new(1f32, -1f32, 1f32));
    matrix *= Matrix4::new_translation(&Vector3::new(0., -h as f32, 0.));

    // translate
    matrix *= Matrix4::new_translation(&Vector3::new(-x, -y, 0.));

    // return it
    let mut t = Mat4([
        0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    ]);
    t.0.clone_from_slice(matrix.as_slice());
    t
}
