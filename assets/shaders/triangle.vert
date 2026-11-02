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

layout(location = 0) out vec3 frag_normal;
layout(location = 1) out vec3 frag_position;
layout(location = 2) out vec4 frag_emissive;

void main() {
    vec4 world_position = pc.model * vec4(position, 1.0);
    frag_position = world_position.xyz;
    frag_normal = normalize(mat3(pc.model) * normal);
    frag_emissive = pc.emissive;
    gl_Position = view_proj * world_position;
}