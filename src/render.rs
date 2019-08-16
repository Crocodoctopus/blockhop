use crate::{camera::camera, io::get_root};
use crossbeam_channel::Receiver;
use ezgl::*;
use glutin::GlWindow;
use std::{collections::HashMap, fs::read_dir, iter::FromIterator};

pub struct RenderState {
    pub sprite_xys: Vec<(f32, f32)>,
    pub sprite_uvs: Vec<(f32, f32)>,
    pub sprite_whs: Vec<(f32, f32)>,
    pub sprite_rghs: Vec<(f32, f32, f32)>,
    pub debug: bool,
    pub wireboxes: Option<Vec<(f32, f32, f32, f32)>>,
    pub rigid_bodies: Option<Vec<(f32, f32)>>,
}

#[derive(Debug)]
pub enum RenderErr {
    Location(u32, u32),
}

pub fn render(
    camw: f32,
    camh: f32,
    window: GlWindow,
    render_recv: Receiver<RenderState>,
) -> Result<(), RenderErr> {
    // build gl context
    unsafe {
        use crate::glutin::GlContext;

        window
            .make_current()
            .map_err(|_| RenderErr::Location(column!(), line!()))?;
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0., 1., 0., 1.);
    }

    // load textures
    let textures: HashMap<String, Texture2D> =
        HashMap::from_iter(read_dir(get_root().join("textures")).unwrap().filter_map(
            |direntry_res| {
                let path = direntry_res.ok()?.path();
                let name = path.file_name()?.to_string_lossy().into_owned();
                let texture = Texture2D::from_file(&path).ok()?;
                println!(
                    "Loaded texture: {:?} as {:?} (0x{:x})",
                    path, name, texture.format
                );
                Some((name, texture))
            },
        ));

    // todo: load all programs
    let sprite_program = ProgramBuilder::new()
        .with(Shader::from_file(&get_root().join("shaders/sprite.vert")).unwrap())
        .with(Shader::from_file(&get_root().join("shaders/sprite.frag")).unwrap())
        .build()
        .unwrap();
    let wireframe_program = ProgramBuilder::new()
        .with(Shader::from_file(&get_root().join("shaders/wireframe.vert")).unwrap())
        .with(Shader::from_file(&get_root().join("shaders/wireframe.frag")).unwrap())
        .build()
        .unwrap();

    loop {
        // get a render state from the update thread
        let render_state = match render_recv.recv() {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        // clear
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // render sprites
        {
            // get number of sprites to render
            let count = render_state.sprite_xys.len();

            // pos
            let pos = render_state.sprite_whs.iter().fold(
                Vec::with_capacity(count * 4),
                |mut v, sprite_wh| {
                    v.push((0., 0.));
                    v.push((sprite_wh.0, 0.));
                    v.push((sprite_wh.0, sprite_wh.1));
                    v.push((0., sprite_wh.1));
                    v
                },
            );
            let vert_data = Buffer::<(f32, f32)>::from(gl::ARRAY_BUFFER, &pos[..]);

            // uv
            let uv = render_state
                .sprite_uvs
                .iter()
                .zip(render_state.sprite_whs.iter())
                .fold(
                    Vec::with_capacity(count * 4),
                    |mut v, (sprite_uv, sprite_wh)| {
                        v.push((sprite_uv.0, sprite_uv.1));
                        v.push((sprite_uv.0 + sprite_wh.0, sprite_uv.1));
                        v.push((sprite_uv.0 + sprite_wh.0, sprite_uv.1 + sprite_wh.1));
                        v.push((sprite_uv.0, sprite_uv.1 + sprite_wh.1));
                        v
                    },
                );
            let uv_data = Buffer::<(f32, f32)>::from(gl::ARRAY_BUFFER, &uv[..]);

            // model_location
            let model_location = render_state.sprite_xys.iter().fold(
                Vec::with_capacity(count * 4),
                |mut v, sprite_xy| {
                    v.push((sprite_xy.0, sprite_xy.1));
                    v.push((sprite_xy.0, sprite_xy.1));
                    v.push((sprite_xy.0, sprite_xy.1));
                    v.push((sprite_xy.0, sprite_xy.1));
                    v
                },
            );
            let model_location_data =
                Buffer::<(f32, f32)>::from(gl::ARRAY_BUFFER, &model_location[..]);

            // model_origin
            let model_origin = render_state.sprite_rghs.iter().fold(
                Vec::with_capacity(count * 4),
                |mut v, sprite_rgh| {
                    v.push((sprite_rgh.1, sprite_rgh.2));
                    v.push((sprite_rgh.1, sprite_rgh.2));
                    v.push((sprite_rgh.1, sprite_rgh.2));
                    v.push((sprite_rgh.1, sprite_rgh.2));
                    v
                },
            );
            let model_origin_data = Buffer::<(f32, f32)>::from(gl::ARRAY_BUFFER, &model_origin[..]);

            // model_rotation
            let model_rotation = render_state.sprite_rghs.iter().fold(
                Vec::with_capacity(count * 4),
                |mut v, sprite_rgh| {
                    v.push(sprite_rgh.0);
                    v.push(sprite_rgh.0);
                    v.push(sprite_rgh.0);
                    v.push(sprite_rgh.0);
                    v
                },
            );
            let model_rotation_data = Buffer::<f32>::from(gl::ARRAY_BUFFER, &model_rotation[..]);

            // ibo
            let ele = (0..count as u32).fold(Vec::with_capacity(count * 6), |mut v, num| {
                v.push(num * 4);
                v.push(num * 4 + 1);
                v.push(num * 4 + 2);
                v.push(num * 4 + 2);
                v.push(num * 4 + 3);
                v.push(num * 4);
                v
            });
            let ibo = Buffer::<u32>::from(gl::ELEMENT_ARRAY_BUFFER, &ele);

            // position and texture
            let pos_transform = camera(0., 0., camw, camh);
            let tex_transform = (752., 254.);

            // draw
            InstantDraw::start_tri_draw(count as u32 * 2, &sprite_program, &ibo)
                .with_buffer(&vert_data, 0)
                .with_buffer(&uv_data, 1)
                .with_buffer(&model_location_data, 2)
                .with_buffer(&model_origin_data, 3)
                .with_buffer(&model_rotation_data, 4)
                .with_texture(&textures["mastercomp.png"], 0)
                .with_uniform(GLSLAny::Mat3(pos_transform), 1)
                .with_uniform(GLSLAny::Vec2(tex_transform), 2)
                .enable_blend(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)
                .draw();
        }

        // render wireboxes
        if let Some(wireboxes) = render_state.wireboxes {
            let count = wireboxes.len();

            // pos
            let pos = wireboxes
                .iter()
                .fold(Vec::with_capacity(count * 4), |mut v, wirebox| {
                    v.push((wirebox.0, wirebox.1));
                    v.push((wirebox.0 + wirebox.2, wirebox.1));
                    v.push((wirebox.0 + wirebox.2, wirebox.1 + wirebox.3));
                    v.push((wirebox.0, wirebox.1 + wirebox.3));
                    v
                });
            let vert_data = Buffer::<(f32, f32)>::from(gl::ARRAY_BUFFER, &pos[..]);

            // color
            let color = wireboxes
                .iter()
                .fold(Vec::with_capacity(count * 4), |mut v, _| {
                    v.push((1., 0., 0., 1.));
                    v.push((1., 0., 0., 1.));
                    v.push((1., 0., 0., 1.));
                    v.push((1., 0., 0., 1.));
                    v
                });
            let color_data = Buffer::<(f32, f32, f32, f32)>::from(gl::ARRAY_BUFFER, &color[..]);

            // bc
            let bc = wireboxes
                .iter()
                .fold(Vec::with_capacity(count * 4), |mut v, _| {
                    v.push((0., 1.));
                    v.push((0., 0.));
                    v.push((1., 0.));
                    v.push((0., 0.));
                    /*v.push(((temp >> 0 & 1) as f32, (temp >> 1 & 1) as f32));
                    v.push(((temp >> 2 & 1) as f32, (temp >> 3 & 1) as f32));
                    v.push(((temp >> 4 & 1) as f32, (temp >> 6 & 1) as f32));
                    v.push(((temp >> 6 & 1) as f32, (temp >> 7 & 1) as f32));*/
                    v
                });
            let bc_data = Buffer::<(f32, f32)>::from(gl::ARRAY_BUFFER, &bc[..]);

            // ibo
            let ele = (0..count as u32).fold(Vec::with_capacity(count * 6), |mut v, num| {
                v.push(num * 4);
                v.push(num * 4 + 1);
                v.push(num * 4 + 2);
                v.push(num * 4 + 2);
                v.push(num * 4 + 3);
                v.push(num * 4);
                v
            });
            let ibo = Buffer::<u32>::from(gl::ELEMENT_ARRAY_BUFFER, &ele);

            // position transform
            let pos_transform = camera(0., 0., camw, camh);

            // draw
            InstantDraw::start_tri_draw(count as u32 * 2, &wireframe_program, &ibo)
                .with_buffer(&vert_data, 0)
                .with_buffer(&color_data, 1)
                .with_buffer(&bc_data, 2)
                .with_uniform(GLSLAny::Mat3(pos_transform), 0)
                .enable_blend(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)
                .draw();
        }

        if let Some(rigid_bodies) = render_state.rigid_bodies {
            let count = rigid_bodies.len();

            // pos
            let pos =
                rigid_bodies
                    .iter()
                    .fold(Vec::with_capacity(count * 4), |mut v, rigid_body| {
                        v.push((rigid_body.0 - 1., rigid_body.1 - 1.));
                        v.push((rigid_body.0 + 1., rigid_body.1 - 1.));
                        v.push((rigid_body.0 + 1., rigid_body.1 + 1.));
                        v.push((rigid_body.0 - 1., rigid_body.1 + 1.));
                        v
                    });
            let vert_data = Buffer::<(f32, f32)>::from(gl::ARRAY_BUFFER, &pos[..]);

            // color
            let color = rigid_bodies
                .iter()
                .fold(Vec::with_capacity(count * 4), |mut v, _| {
                    v.push((0., 1., 0., 1.));
                    v.push((0., 1., 0., 1.));
                    v.push((0., 1., 0., 1.));
                    v.push((0., 1., 0., 1.));
                    v
                });
            let color_data = Buffer::<(f32, f32, f32, f32)>::from(gl::ARRAY_BUFFER, &color[..]);

            // bc
            let bc = rigid_bodies
                .iter()
                .fold(Vec::with_capacity(count * 4), |mut v, _| {
                    v.push((0., 1.));
                    v.push((0., 0.));
                    v.push((1., 0.));
                    v.push((0., 0.));
                    /*v.push(((temp >> 0 & 1) as f32, (temp >> 1 & 1) as f32));
                    v.push(((temp >> 2 & 1) as f32, (temp >> 3 & 1) as f32));
                    v.push(((temp >> 4 & 1) as f32, (temp >> 6 & 1) as f32));
                    v.push(((temp >> 6 & 1) as f32, (temp >> 7 & 1) as f32));*/
                    v
                });
            let bc_data = Buffer::<(f32, f32)>::from(gl::ARRAY_BUFFER, &bc[..]);

            // ibo
            let ele = (0..count as u32).fold(Vec::with_capacity(count * 6), |mut v, num| {
                v.push(num * 4);
                v.push(num * 4 + 1);
                v.push(num * 4 + 2);
                v.push(num * 4 + 2);
                v.push(num * 4 + 3);
                v.push(num * 4);
                v
            });
            let ibo = Buffer::<u32>::from(gl::ELEMENT_ARRAY_BUFFER, &ele);

            // position transform
            let pos_transform = camera(0., 0., camw, camh);

            // draw
            InstantDraw::start_tri_draw(count as u32 * 2, &wireframe_program, &ibo)
                .with_buffer(&vert_data, 0)
                .with_buffer(&color_data, 1)
                .with_buffer(&bc_data, 2)
                .with_uniform(GLSLAny::Mat3(pos_transform), 0)
                .enable_blend(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)
                .draw();
        }

        // swap buffer
        window
            .swap_buffers()
            .map_err(|_| RenderErr::Location(column!(), line!()))?;
    }
}
