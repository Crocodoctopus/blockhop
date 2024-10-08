#version 430 core

layout(location = 0) in vec2 vert_xy; // world position
layout(location = 1) in vec4 vert_rgba; // color
layout(location = 2) in vec2 bc;

layout(location = 0) uniform mat3 view_matrix;

out vec2 frag_uv;
out vec4 frag_rgba;
out vec2 vbc;

void main() { 	
	gl_Position = vec4((view_matrix * vec3(vert_xy, 1)).xy, 0, 1);

	frag_rgba = vert_rgba;
	vbc = bc;
}