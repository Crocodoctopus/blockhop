#version 430

in vec2 frag_uv;

layout(location = 0) uniform sampler2D tex;
layout(location = 2) uniform vec2 tex_size;

out vec4 rgba;

void main() {
	rgba = texture(tex, frag_uv/tex_size);
}