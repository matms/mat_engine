#version 450

layout(location=0) in vec2 a_position;
layout(location=1) in vec2 a_tex_coords;

layout(location=2) in mat4 a_model_mat; // Occupies locations 2 through 5 inclusive.

layout(location=0) out vec2 v_tex_coords;

layout(set=1, binding=0) 
uniform TestUniform{
    mat4 u_proj_mat;
};


void main() {
    v_tex_coords = a_tex_coords;
    gl_Position =  u_proj_mat * a_model_mat * vec4(a_position, 0.0, 1.0);
}

 

 