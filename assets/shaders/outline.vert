#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

struct Light {
    vec3 position;
    float _pad0;
    vec3 color;
    float power;
};

layout(std140, set = 0, binding = 0) uniform Scene {
    mat4 view;
    mat4 projection;
    mat4 inv_view_proj;
    vec3 camera_position;
    float _pad0;
    uint light_count;
    uint _pad1a;
    uint _pad1b;
    uint _pad1c;
    Light lights[8];
    vec3 sky_top;
    float _pad2;
    vec3 sky_bottom;
    float _pad3;
};

layout(push_constant) uniform Push {
    mat4 model;
    vec4 emissive;
} pc;

const float OUTLINE_WIDTH = 0.05;

void main() {
    mat3 m3 = mat3(pc.model);

    float scale = (length(m3[0]) + length(m3[1]) + length(m3[2])) / 3.0;

    vec3 world_normal = normalize(transpose(inverse(m3)) * normal);

    vec4 world_position = pc.model * vec4(position, 1.0);
    world_position.xyz += world_normal * OUTLINE_WIDTH * scale;

    gl_Position = projection * view * world_position;
}