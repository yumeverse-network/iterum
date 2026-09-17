#version 460

layout(location = 0) in vec3 frag_normal;
layout(location = 1) in vec3 frag_position;

layout(std140, set = 0, binding = 0) uniform Scene {
    mat4 model;
    mat4 view;
    mat4 projection;

    vec3 light_position;
    float _padding1;

    vec3 light_color;
    float light_intensity;
};

layout(location = 0) out vec4 f_color;

void main() {
    vec3 normal = normalize(frag_normal);

    vec3 light_direction = light_position - frag_position;
    float distance = length(light_direction);

    light_direction = normalize(light_direction);

    float diffuse = max(dot(normal, light_direction), 0.0);

    float attenuation = 1.0 / max(distance * distance, 0.01);

    vec3 lighting =
        light_color *
        diffuse *
        light_intensity *
        attenuation;

    vec3 base_color = vec3(1.0, 0.1, 0.6);

    f_color = vec4(base_color * lighting, 1.0);
}