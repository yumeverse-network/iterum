#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(std140, set = 0, binding = 0) uniform Scene {
    mat4 model;
    mat4 view;
    mat4 projection;

    vec3 light_position;
    float _padding1;

    vec3 light_color;
    float light_intensity;

    vec3 camera_position;
    float _padding2;
};

layout(location = 0) out vec3 frag_normal;
layout(location = 1) out vec3 frag_position;

void main() {
    vec4 world_position = model * vec4(position, 1.0);

    frag_position = world_position.xyz;
    frag_normal = normalize(mat3(model) * normal);

    vec4 clip = projection * view * world_position;

    gl_Position = clip;
}