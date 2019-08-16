#version 430 core

layout(location = 0) in vec2 vert_xy;
layout(location = 1) in vec2 vert_uv; // in pixels
layout(location = 2) in vec2 model_location;
layout(location = 3) in vec2 model_origin;
layout(location = 4) in float model_rotation;

layout(location = 1) uniform mat3 view_matrix;

out vec2 frag_uv;

mat3 trans2d(vec2 f) {
	return mat3(
		1, 0, 0,
		0, 1, 0,
		f.x, f.y, 1);
}

mat3 rot2d(float r) {
	return mat3(
		cos(r), sin(r), 0,
		-sin(r), cos(r), 0,
		0, 0, 1);
}

void main() { 	
	// build the model matrix in the shader
	mat3 model_matrix = trans2d(model_location) * rot2d(model_rotation) * trans2d(model_origin);

	//
	gl_Position = vec4((view_matrix * model_matrix * vec3(vert_xy, 1)).xy, 0, 1);

	//
	frag_uv = vert_uv;
}









/*
#version 430 core

layout(location = 0) in vec2 vert_xy; // world position
layout(location = 1) in vec2 vert_uv; // texture position
layout(location = 2) in vec2 vert_gh; // origin
layout(location = 3) in float vert_r; // rotation

layout(location = 1) uniform mat4 view_transform;

out vec2 frag_uv;

mat3 trans2d(vec2 f) {
	return mat3(
		vec3(1, 0, f.x),
		vec3(0, 1, f.y),
		vec3(0, 0, 1)
	);
}

mat3 rot2d(float r) {
	return mat3(
		vec3(cos(r), -sin(r), 0),
		vec3(sin(r), cos(r), 0),
		vec3(0, 0, 1)
	);
}

void main() { 	
	mat3 model_transform = rot2d(vert_r) * trans2d(vert_gh);
	gl_Position = vec4((view_transform * model_transform * vec3(vert_xy, 1)).xy, 0, 1);

	frag_uv = vert_uv;
}
*/