#version 460

layout(location = 0) in vec3 frag_normal;
layout(location = 1) in vec3 frag_position;
layout(location = 2) in vec4 frag_emissive;

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

layout(location = 0) out vec4 f_color;
layout(depth_any) out float gl_FragDepth;

void main() {
    vec3 color;

    if (frag_emissive.a > 0.0) {
        color = frag_emissive.rgb * frag_emissive.a;
    } else {
        vec3 N = normalize(frag_normal);
        vec3 V = normalize(-frag_position);

        vec3 base_color = vec3(1.0, 0.1, 0.6);
        color = base_color * vec3(0.05);

        for (uint i = 0u; i < light_count; i++) {
            Light l = lights[i];
            vec3 to_light = l.position - frag_position;
            float d = length(to_light);
            vec3 L = to_light / max(d, 1e-4);
            vec3 H = normalize(L + V);

            float atten  = l.power / max(d * d, 1.0);
            float NdotL  = max(dot(N, L), 0.0);
            float NdotH  = max(dot(N, H), 0.0);

            color += base_color * NdotL * l.color * atten;
            color += pow(NdotH, 64.0) * 0.4 * l.color * atten;
        }
    }

    color = color / (color + vec3(1.0));

    const float FAR = 1.0e13;
    const float Fcoef = 1.0 / log2(FAR + 1.0);
    float dist = length(frag_position);
    gl_FragDepth = log2(max(1e-6, 1.0 + dist)) * Fcoef;

    f_color = vec4(color, 1.0);
}