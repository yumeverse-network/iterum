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

    vec3 camera_position;
    float _padding2;
};

layout(location = 0) out vec4 f_color;
layout(depth_any) out float gl_FragDepth;

void main() {
    vec3 normal = normalize(frag_normal);

    // Directional: pretend the light is at a fixed direction from the surface.
    // This ignores light_position and uses a fixed sun direction instead.
    vec3 light_dir = normalize(vec3(-0.5, 0.5, 1.0));

    float diffuse = max(dot(normal, light_dir), 0.0);
    float ambient = 0.15;

    vec3 base_color = vec3(1.0, 0.1, 0.6);

    vec3 lighting = light_color * (diffuse * 0.85 + ambient);

    vec3 final_color = base_color * lighting;

    // Reinhard tone map — compress to [0,1] without clipping
    final_color = final_color / (final_color + vec3(1.0));

    const float FAR = 1.0e13;
    const float Fcoef = 1.0 / log2(FAR + 1.0);
    float dist = length(frag_position - camera_position);
    gl_FragDepth = log2(max(1e-6, 1.0 + dist)) * Fcoef;

    f_color = vec4(final_color, 1.0);
}