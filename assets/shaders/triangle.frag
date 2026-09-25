#version 460

layout(location = 0) in vec3 frag_normal;
layout(location = 1) in vec3 frag_position;

layout(std140, set = 0, binding = 0) uniform Scene {
    //mat4 model;
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
    vec3 N = normalize(frag_normal);
    vec3 V = normalize(-frag_position);
    vec3 L = normalize(light_position - frag_position);
    vec3 H = normalize(L + V);

    float NdotL = max(dot(N, L), 0.0);
    float NdotH = max(dot(N, H), 0.0);

    vec3 diffuse  = NdotL * light_color * light_intensity;
    vec3 specular = pow(NdotH, 64.0) * 0.4 * light_color * light_intensity;
    vec3 ambient  = vec3(0.05);

    vec3 base_color = vec3(1.0, 0.1, 0.6);
    vec3 final_color = base_color * (ambient + diffuse) + specular;
    final_color = final_color / (final_color + vec3(1.0));

    const float FAR = 1.0e13;
    const float Fcoef = 1.0 / log2(FAR + 1.0);
    float dist = length(frag_position);
    gl_FragDepth = log2(max(1e-6, 1.0 + dist)) * Fcoef;

    f_color = vec4(final_color, 1.0);
}