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
    mat4 inv_view_proj;
    vec3 camera_position;
    float _pad0;
    uint light_count;
    uint _pad1a;
    uint _pad1b;
    uint _pad1c;
    Light lights[MAX_LIGHTS];
    vec3 sky_top;
    float _pad2;
    vec3 sky_bottom;
    float _pad3;
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

        /*vec3 base_color = vec3(1.0, 0.1, 0.6);
        color = base_color * vec3(0.05);*/

        vec3 base_color = vec3(1.0, 0.1, 0.6);
        
        vec3 sky_luma = vec3(dot(sky_top, vec3(0.2126, 0.7152, 0.0722)));
        vec3 ambient_sky = mix(sky_top, sky_luma, 0.75) * 1.6;      // pale, bright
        
        vec3 ground_luma = vec3(dot(sky_bottom, vec3(0.2126, 0.7152, 0.0722)));
        vec3 ambient_ground = mix(sky_bottom, ground_luma, 0.6) * 0.4;
        
        float hemi = N.y * 0.5 + 0.5;
        vec3 ambient_color = mix(ambient_ground, ambient_sky, hemi);
        
        color = base_color * ambient_color;

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