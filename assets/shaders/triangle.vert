#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

const int MAX_LIGHTS = 8;

struct Light {
    vec3 position;
    float _pad0;
    vec3 color;
    float power;
};

layout(std140, set = 0, binding = 0) uniform Scene {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
    float _pad0;
    uint light_count;
    uint _pad1a;
    uint _pad1b;
    uint _pad1c;
    Light lights[MAX_LIGHTS];
};

layout(push_constant) uniform Push {
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
    gl_Position = projection * view * world_position;
}