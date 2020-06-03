#version 450

layout(constant_id = 0) const float aspect_ratio = 1.0;

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 pass_color;


void main() {
    // use a marker for alignmentyttt
    gl_Position = vec4(((position.x+1.0)/aspect_ratio)-1.0, position.y, 0.0, 1.0);
    pass_color = color;
}
