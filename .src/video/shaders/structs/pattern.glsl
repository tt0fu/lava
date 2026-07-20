#ifndef PATTERN
#define PATTERN

#include "../lib/oklab.glsl"
#include "../lib/noise.glsl"

struct Pattern {
    vec3 color;

    bool use_rainbow;

    float lightness;
    float chroma;
    float scale;
    float repeats;
    float pattern_speed;
    float scroll_speed;
};

vec3 get_color(Pattern p, vec2 uv, float aspect_ratio, float chrono) {
    if (p.use_rainbow) {
        return lch_srgb(vec3(p.lightness, p.chroma, fract(fbm3(uv * vec2(aspect_ratio, 1.0) * p.scale, chrono * p.pattern_speed) * p.repeats) + chrono * p.scroll_speed));
    } else {
        return p.color;
    }
}

#endif
