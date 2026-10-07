#version 460

layout(location = 0) in vec2 v_ndc;

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
};

layout(push_constant) uniform SkyPush {
    vec3 sun_direction;
    float sun_intensity;
    vec3 sun_color;
    float _pad0;
    vec3 sky_top;
    float _pad1;
    vec3 sky_bottom;
    float _pad2;
    vec4 _pad3;
    float star_density;
    float star_brightness;
    float star_seed;
    float _pad4;
} pc;

layout(location = 0) out vec4 f_color;

float hash13(vec3 p) {
    p = fract(p * vec3(0.1031, 0.1030, 0.0973));
    p += dot(p, p.yzx + 33.33);
    p += dot(p.yzy, p + 45.32);
    return fract((p.x + p.y) * p.z);
}

float hash21(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

vec3 hash23(vec2 p) {
    return vec3(
        hash21(p),
        hash21(p + vec2(17.1, 23.7)),
        hash21(p + vec2(91.3, 55.2))
    );
}

vec3 hash33(vec3 p) {
    p = fract(p * vec3(0.1031, 0.1030, 0.0973));
    p += dot(p, p.yzx + 33.33);
    p += dot(p.yzy, p + 45.32);
    return fract((p.xxy + p.yxx) * p.zyx);
}

vec3 star_layer(vec3 dir, float density, float brightness, float seed) {
    vec3 p = dir * density + vec3(seed);
    vec3 base = floor(p);

    //float aa = length(fwidth(p)) * 0.5;
    float aa = length(fwidth(dir)) * density * 0.5;

    vec3 result = vec3(0.0);

    for (int x = -1; x <= 1; x++)
    for (int y = -1; y <= 1; y++)
    for (int z = -1; z <= 1; z++) {
        vec3 cell = base + vec3(x, y, z);
        vec3 rnd  = hash33(cell);

        if (hash13(cell + 11.7) > 0.35) continue;

        vec3 star_pos = cell + rnd;
        float d = length(p - star_pos);

        //float radius = max(0.03 + rnd.z * 0.05, aa);
        float radius = min(max(0.03 + rnd.z * 0.05, aa), 0.12);
        float star = smoothstep(radius, 0.0, d);
        star *= star;

        float tint_val = hash13(cell + 5.0);
        vec3 tint = mix(vec3(0.75, 0.85, 1.0), vec3(1.0, 0.9, 0.7), tint_val);

        float b = 0.4 + 0.6 * hash13(cell + 29.1);

        result += tint * star * b * brightness;
    }
    return result;
}

void main() {
    vec4 far = inv_view_proj * vec4(v_ndc, 1.0, 1.0);
    vec3 dir = normalize(far.xyz);

    float t = dir.y * 0.5 + 0.5;
    vec3 color = mix(pc.sky_bottom, pc.sky_top, t);

    vec3  sun_dir = normalize(pc.sun_direction);
    float dist    = length(dir - sun_dir);
    float sun_up  = smoothstep(-0.20, 0.05, sun_dir.y);

    float bloom = exp(-dist * 28.0) * 0.20
                + exp(-dist * 10.0) * 0.04;
    bloom *= 1.0 - smoothstep(0.15, 0.30, dist);
    bloom *= sun_up;

    vec3 stars = star_layer(dir, pc.star_density,       pc.star_brightness,       pc.star_seed)
               + star_layer(dir, pc.star_density * 2.7, pc.star_brightness * 0.5, pc.star_seed + 100.0);
    stars *= 1.0 - clamp(bloom * 6.0, 0.0, 1.0);
    color += stars;

    float disk_radius = 0.03;
    float aa   = fwidth(dist) * 1.5;
    float disk = 1.0 - smoothstep(disk_radius - aa, disk_radius + aa, dist);

    color += pc.sun_color * bloom * pc.sun_intensity;
    color  = mix(color, pc.sun_color * pc.sun_intensity * 2.0, disk * sun_up);

    f_color = vec4(color, 1.0);
}
