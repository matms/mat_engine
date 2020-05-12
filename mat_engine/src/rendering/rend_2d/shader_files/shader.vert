#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;

layout(location=0) out vec2 v_tex_coords;

layout(set=1, binding=0) 
uniform TestUniform{
    float num;
};


void main() {
    v_tex_coords = a_tex_coords;

    vec4 pos = vec4(a_position, 1.0);
    pos.x += num;
    gl_Position = pos;
}

 

 