#version 460

layout(location = 0) in vec2 v_ndc;

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

vec3 hash33(vec3 p) {
    p = fract(p * vec3(0.1031, 0.1030, 0.0973));
    p += dot(p, p.yzx + 33.33);
    p += dot(p.yzy, p + 45.32);
    return fract((p.xxy + p.yxx) * p.zyx);
}

vec3 star_layer(vec3 dir, float density, float brightness, float seed) {
    vec3 p = dir * density + vec3(seed);
    vec3 base = floor(p);

    float aa = length(fwidth(dir)) * density * 0.5;

    vec3 result = vec3(0.0);

    for (int x = -1; x <= 1; x++)
    for (int y = -1; y <= 1; y++)
    for (int z = -1; z <= 1; z++) {
        vec3 cell = base + vec3(x, y, z);
        vec3 rnd = hash33(cell);

        if (hash13(cell + 11.7) > 0.35) continue;

        vec3 star_pos = cell + rnd;
        float d = length(p - star_pos);

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

    vec3 sun_dir = vec3(0.0, 1.0, 0.0);
    vec3 sun_color = vec3(0.0);
    float sun_intensity = 0.0;
    if (light_count.x > 0u) {
        vec4 lp = lights[0].position;
        if (length(lp.xyz) > 1e-6) {
            sun_dir = normalize(lp.xyz);
        }
        sun_color = lights[0].color_power.rgb;
        sun_intensity = lights[0].color_power.w;
    }

    float star_density = space.x;
    float star_brightness = space.y;
    float star_seed = space.z;

    float t = dir.y * 0.5 + 0.5;
    vec3 color = mix(sky_bottom.rgb, sky_top.rgb, t);

    float dist = length(dir - sun_dir);
    float sun_up = smoothstep(-0.20, 0.05, sun_dir.y);

    float bloom = exp(-dist * 28.0) * 0.20
                + exp(-dist * 10.0) * 0.04;
    bloom *= 1.0 - smoothstep(0.15, 0.30, dist);
    bloom *= sun_up;

    vec3 stars = star_layer(dir, star_density,       star_brightness,       star_seed)
               + star_layer(dir, star_density * 2.7, star_brightness * 0.5, star_seed + 100.0);
    stars *= 1.0 - clamp(bloom * 6.0, 0.0, 1.0);
    color += stars;

    float disk_radius = 0.03;
    float aa = fwidth(dist) * 1.5;
    float disk = 1.0 - smoothstep(disk_radius - aa, disk_radius + aa, dist);

    color += sun_color * bloom * sun_intensity;
    if (sun_intensity > 0.0) {
        color = mix(color, sun_color * sun_intensity * 2.0, disk * sun_up);
    }

    f_color = vec4(color, 1.0);
}