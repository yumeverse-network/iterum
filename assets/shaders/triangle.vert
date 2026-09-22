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
};

layout(location = 0) out vec3 frag_normal;
layout(location = 1) out vec3 frag_position;

// Must match Camera::far
const float FAR = 1.0e13;

void main() {
    vec4 world_position = model * vec4(position, 1.0);

    frag_position = world_position.xyz;
    frag_normal = mat3(model) * normal;

    vec4 clip = projection * view * world_position;

    // Vulkan log depth: maps [0, FAR] -> [0, 1] in NDC.
    // (OpenGL's version multiplies by 2 and subtracts 1 — that clips near geometry in Vulkan.)
    const float Fcoef = 1.0 / log2(FAR + 1.0);
    clip.z = log2(max(1e-6, 1.0 + clip.w)) * Fcoef * clip.w;

    gl_Position = clip;
}