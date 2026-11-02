#version 460

layout(location = 0) in vec3 frag_normal;
layout(location = 1) in vec3 frag_position;
layout(location = 2) in vec4 frag_emissive;

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

layout(location = 0) out vec4 f_color;

void main() {
    vec3 color;

    if (frag_emissive.a > 0.0) {
        color = frag_emissive.rgb * frag_emissive.a;
    } else {
        vec3 N = normalize(frag_normal);
        vec3 V = normalize(-frag_position);

        vec3 base_color = vec3(1.0, 0.1, 0.6);

        vec3 sky_luma = vec3(dot(sky_top.rgb, vec3(0.2126, 0.7152, 0.0722)));
        vec3 ambient_sky = mix(sky_top.rgb, sky_luma, 0.75) * 1.6;

        vec3 ground_luma = vec3(dot(sky_bottom.rgb, vec3(0.2126, 0.7152, 0.0722)));
        vec3 ambient_ground = mix(sky_bottom.rgb, ground_luma, 0.6) * 0.4;

        float hemi = N.y * 0.5 + 0.5;
        vec3 ambient_color = mix(ambient_ground, ambient_sky, hemi);

        color = base_color * ambient_color;

        for (uint i = 0u; i < light_count.x; i++) {
            Light l = lights[i];
            vec3 lpos = l.position.xyz;
            vec3 lcol = l.color_power.rgb;
            float lpow = l.color_power.w;

            vec3 to_light = lpos - frag_position;
            float d = length(to_light);
            vec3 L = to_light / max(d, 1e-4);
            vec3 H = normalize(L + V);

            float atten = lpow / max(d * d, 1.0);
            float NdotL = max(dot(N, L), 0.0);
            float NdotH = max(dot(N, H), 0.0);

            color += base_color * NdotL * lcol * atten;
            color += pow(NdotH, 64.0) * 0.4 * lcol * atten;
        }
    }

    color = color / (color + vec3(1.0));

    const float FAR = 1.0e13;
    const float Fcoef = 1.0 / log2(FAR + 1.0);
    float dist = length(frag_position);
    gl_FragDepth = log2(max(1e-6, 1.0 + dist)) * Fcoef;

    f_color = vec4(color, 1.0);
}