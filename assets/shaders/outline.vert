#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

const int MAX_LIGHTS = 8;

struct Light {
    vec4 position;
    vec4 color_power;
};

layout(std140, set = 0, binding = 0) uniform Scene {
    mat4 view_proj;
    mat4 inv_view_proj;
    vec4 viewport;
    vec4 sky_top;
    vec4 sky_bottom;
    uvec4 light_count;
    Light lights[MAX_LIGHTS];
    vec4 space;
};

layout(std140, set = 1, binding = 0) uniform Model {
    mat4 model;
    vec4 emissive;
} pc;

const float OUTLINE_WIDTH = 0.05;

void main() {
    mat3 m3 = mat3(pc.model);

    float scale = (length(m3[0]) + length(m3[1]) + length(m3[2])) / 3.0;

    vec3 world_normal = normalize(m3 * normal);

    vec4 world_position = pc.model * vec4(position, 1.0);
    world_position.xyz += world_normal * OUTLINE_WIDTH * scale;

    gl_Position = view_proj * world_position;
}