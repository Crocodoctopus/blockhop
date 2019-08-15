#version 430 core

layout(location = 0) in vec2 vert_xy; // world position
layout(location = 1) in vec4 vert_rgba; // color
layout(location = 2) in vec2 bc;

layout(location = 0) uniform mat4 pos_transform;

out vec2 frag_uv;
out vec4 frag_rgba;
out vec2 vbc;

void main() { 	
	gl_Position = pos_transform * vec4(vert_xy, 0, 1);

	frag_rgba = vert_rgba;
	vbc = bc;
}