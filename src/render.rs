use crate::{
    io::get_root,
};
use crossbeam_channel::Receiver;
use ezgl::*;
use glutin::GlWindow;
use std::{
    collections::{HashMap},
    fs::read_dir,
    iter::FromIterator,
};

pub struct RenderState {
    pub debug: bool,
}

#[derive(Debug)]
pub enum RenderErr {
    Location(u32, u32),
}

pub fn render(window: GlWindow, render_recv: Receiver<RenderState>) -> Result<(), RenderErr> {
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
                println!("Loaded texture: {:?} as {:?}", path, name);
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

        // render

        // render hitbox wireframes
        /*if render_state.debug {
            let count = render_state.sprites.len();

            // pos
            let pos = render_state.hitboxes.values().fold(
                Vec::with_capacity(count * 4),
                |mut v, hitbox| {
                    v.push((hitbox.x, hitbox.y));
                    v.push((hitbox.x + hitbox.w, hitbox.y));
                    v.push((hitbox.x + hitbox.w, hitbox.y + hitbox.h));
                    v.push((hitbox.x, hitbox.y + hitbox.h));
                    v
                },
            );
            let vert_data = Buffer::<(f32, f32)>::from(gl::ARRAY_BUFFER, &pos[..]);

            // color
            let color =
                render_state
                    .hitboxes
                    .values()
                    .fold(Vec::with_capacity(count * 4), |mut v, _| {
                        v.push((1., 0., 0., 1.));
                        v.push((1., 0., 0., 1.));
                        v.push((1., 0., 0., 1.));
                        v.push((1., 0., 0., 1.));
                        v
                    });
            let color_data = Buffer::<(f32, f32, f32, f32)>::from(gl::ARRAY_BUFFER, &color[..]);

            // bc
            let bc =
                render_state
                    .hitboxes
                    .values()
                    .fold(Vec::with_capacity(count * 4), |mut v, _| {
                        v.push((1., 0., 0.));
                        v.push((0., 1., 0.));
                        v.push((0., 1., 1.));
                        v.push((0., 1., 0.));
                        v
                    });
            let bc_data = Buffer::<(f32, f32, f32)>::from(gl::ARRAY_BUFFER, &bc[..]);

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
            let pos_transform = camera(0., 0., 1280., 720.);

            // draw
            InstantDraw::start_tri_draw(count as u32 * 2, &wireframe_program, &ibo)
                .with_buffer(&vert_data, 0)
                .with_buffer(&color_data, 1)
                .with_buffer(&bc_data, 2)
                .with_uniform(GLSLAny::Mat4(pos_transform), 0)
                .enable_blend(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)
                .draw();
        }*/

        // swap buffer
        window
            .swap_buffers()
            .map_err(|_| RenderErr::Location(column!(), line!()))?;
    }
}
