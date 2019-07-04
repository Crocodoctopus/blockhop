#version 430 core

layout(location = 0) in vec2 vert_xy; // world position
layout(location = 1) in vec2 vert_uv; // texture position

layout(location = 1) uniform mat4 pos_transform;

out vec2 frag_uv;

void main() { 	
	gl_Position = pos_transform * vec4(vert_xy, 0, 1);

	frag_uv = vert_uv;
}