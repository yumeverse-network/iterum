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

void main() {
    vec4 world_position = pc.model * vec4(position, 1.0);
    vec3 world_normal = normalize(
        transpose(inverse(mat3(pc.model))) * normal
    );

    vec4 clip_position = projection * view * world_position;
    vec4 clip_normal = projection * view * vec4(world_normal, 0.0);

    vec2 screen_normal =
        clip_normal.xy * clip_position.w -
        clip_position.xy * clip_normal.w;

    float normal_length = length(screen_normal);

    if (normal_length > 0.00001) {
        float aspect = projection[1][1] / projection[0][0];
        vec2 pixel_normal = normalize(
            vec2(screen_normal.x * aspect, screen_normal.y)
        );

        vec2 offset = vec2(pixel_normal.x / aspect, pixel_normal.y) * 0.009;
        clip_position.xy += offset * clip_position.w;
    }

    gl_Position = clip_position;
}