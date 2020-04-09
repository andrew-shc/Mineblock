#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 txtr_crd;

layout(location = 0) out vec2 txl_crd;

layout(set = 1, binding = 0) uniform Matrix {
    mat4 proj;
    mat4 view;
    mat4 world;
} matrix;

void main() {
    gl_Position = matrix.proj * matrix.view * matrix.world * vec4(position, 1.0);
    txl_crd = txtr_crd;
}