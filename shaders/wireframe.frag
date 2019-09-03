#version 430

in vec4 frag_rgba;
in vec2 vbc;

out vec4 rgba;

void main() {
	if(any(lessThan(vbc, vec2(0.05)))){
	    rgba = frag_rgba;
	}
	else{
	    rgba = frag_rgba * vec4(1., 1., 1., .5);
	}
}
