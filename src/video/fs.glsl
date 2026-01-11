#version 450

const uint SAMPLE_COUNT = 4096u;
const uint BIN_COUNT = 512u;
const float SAMPLE_COUNT_F = float(SAMPLE_COUNT);
const float BIN_COUNT_F = float(BIN_COUNT);
const float SAMPLE_RATE = 48000.0;
const float LOWEST_FREQUENCY = SAMPLE_RATE / SAMPLE_COUNT_F;
const float EXP_BINS = floor(BIN_COUNT_F / log2(SAMPLE_RATE / (2.0 * LOWEST_FREQUENCY)));

layout(location = 0) in vec2 UV;
layout(location = 0) out vec4 COLOR;

layout(set = 0, binding = 0) uniform Data {
    float scale_x;
    uint samples_start;
    float samples_data[SAMPLE_COUNT];
    float period;
    float focus;
    float center_sample;
    float bass;
    float chrono;

    float line_width;
};

const mat3 fwdA = mat3(
        vec3(1.0, 1.0, 1.0),
        vec3(0.3963377774, -0.1055613458, -0.0894841775),
        vec3(0.2158037573, -0.0638541728, -1.2914855480)
    );
const mat3 fwdB = mat3(
        vec3(4.0767245293, -1.2681437731, -0.0041119885),
        vec3(-3.3072168827, 2.6093323231, -0.7034763098),
        vec3(0.2307590544, -0.3411344290, 1.7068625689)
    );

const mat3 invA = mat3(
        vec3(0.2104542553, 1.9779984951, 0.0259040371),
        vec3(0.7936177850, -2.4285922050, 0.7827717662),
        vec3(-0.0040720468, 0.4505937099, -0.8086757660)
    );

const mat3 invB = mat3(
        vec3(0.4121656120, 0.2118591070, 0.0883097947),
        vec3(0.5362752080, 0.6807189584, 0.2818474174),
        vec3(0.0514575653, 0.1074065790, 0.6302613616)
    );

vec3 lsrgb_oklab(vec3 lsrgb) {
    vec3 lms = invB * lsrgb;
    return invA * (sign(lms) * pow(abs(lms), vec3(0.3333333333333)));
}

vec3 oklab_lsrgb(vec3 oklab) {
    vec3 lms = fwdA * oklab;
    return fwdB * (lms * lms * lms);
}

#define TWO_PI 6.28318530718

vec3 lch_oklab(vec3 lch) {
    float h = lch.b * TWO_PI;
    return vec3(lch.r, lch.g * cos(h), lch.g * sin(h));
}

vec3 oklab_lch(vec3 lab) {
    float a = lab.g;
    float b = lab.b;
    return vec3(lab.r, sqrt(a * a + b * b), atan(b / a) / TWO_PI);
}

vec3 lch_lsrgb(vec3 lch) {
    return oklab_lsrgb(lch_oklab(lch));
}

vec3 lsrgb_lch(vec3 lsrgb) {
    return oklab_lch(lsrgb_oklab(lsrgb));
}

vec3 hueshift(vec3 lsrgb, float shift) {
    vec3 lch = lsrgb_lch(lsrgb);
    lch.b = fract(lch.b + shift + 1.0);
    return lch_lsrgb(lch);
}

vec3 lsrgb_srgb(vec3 lsrgb) {
    vec3 xlo = 12.92 * lsrgb;
    vec3 xhi = 1.055 * pow(lsrgb, vec3(0.4166666666666667)) - 0.055;
    return mix(xlo, xhi, step(vec3(0.0031308), lsrgb));
}

vec3 srgb_lsrgb(vec3 srgb) {
    vec3 xlo = srgb / 12.92;
    vec3 xhi = pow((srgb + 0.055) / 1.055, vec3(2.4));
    return mix(xlo, xhi, step(vec3(0.04045), srgb));
}

vec3 lch_srgb(vec3 lch) {
    return lsrgb_srgb(lch_lsrgb(lch));
}

vec3 srgb_lch(vec3 srgb) {
    return lsrgb_lch(srgb_lsrgb(srgb));
}

float rand(vec2 n) {
    return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 12.345);
}

float noise(vec2 p) {
    vec2 ip = floor(p);
    vec2 u = fract(p);
    u = u * u * (3.0 - 2.0 * u);

    float res = mix(
            mix(rand(ip), rand(ip + vec2(1.0, 0.0)), u.x),
            mix(rand(ip + vec2(0.0, 1.0)), rand(ip + vec2(1.0, 1.0)), u.x), u.y);
    return res * res;
}

const mat2 mtx = mat2(vec2(0.80, 0.60), vec2(-0.60, 0.80));

float fbm(vec2 p, float time) {
    float f = 0.0;
    f += 0.500000 * noise(p + cos(time));
    p = mtx * p * 2.01;
    f += 0.250000 * noise(p + sin(time));
    p = mtx * p;
    f += 0.125000 * noise(p);
    return f / 0.875;
}

float pattern(vec2 p, float time) {
    //return fbm(p, time);
    return fbm(p + fbm(p + fbm(p, time), time), time);
}

float get_raw_sample(int sample_index) {
    if (sample_index < 0) {
        sample_index += int(period * ceil(float(-sample_index) / period));
    }
    if (sample_index >= int(SAMPLE_COUNT)) {
        sample_index -= int(period * ceil(float(sample_index - int(SAMPLE_COUNT) + 1) / period));
    }
    return samples_data[(uint(sample_index) + samples_start) % SAMPLE_COUNT];
}

float get_sample(float sample_index) {
    return mix(
        get_raw_sample(int(floor(sample_index))),
        get_raw_sample(int(ceil(sample_index))),
        fract(sample_index)
    );
}

float fade(float dist) {
    float x = clamp(dist, 0, 1);
    return 1.0 - (x * x);
}

float pseudo_cross(vec2 a, vec2 b) {
    return a.x * b.y - b.x * a.y;
}

float point_to_segment(vec2 a, vec2 b, vec2 p) {
    vec2 pa = a - p;
    vec2 pb = b - p;
    vec2 ab = b - a;
    if (dot(ab, -pa) < 0.0 || dot(-ab, -pb) < 0.0) {
        return min(length(pa), length(pb));
    }
    if (length(ab) < 1e-12) {
        return length(pa);
    }
    return abs(pseudo_cross(pa, pb)) / length(ab);
}

vec2 sample_point(float sample_index) {
    return vec2(sample_index / SAMPLE_COUNT_F * scale_x,
        get_sample(sample_index) * 0.5 + 0.5);
}

float wave_distance(float sample_index, float sample_height) {
    vec2 target = vec2(sample_index / SAMPLE_COUNT_F * scale_x, sample_height);
    float start_index = floor(sample_index - line_width);
    float end_index = ceil(sample_index + line_width);
    float mn = 100000.0;
    vec2 prev = sample_point(start_index);
    for (float index = start_index + 1.0; index <= end_index; index++) {
        vec2 cur = sample_point(index);
        mn = min(mn, point_to_segment(prev, cur, target));
        prev = cur;
    }
    return mn;
}

void main() {
    float sample_index = UV.x * float(SAMPLE_COUNT);
    float dist = wave_distance(sample_index + center_sample - float(SAMPLE_COUNT) * focus, 1.0 - UV.y); //+ center_sample - float(SAMPLE_COUNT) * focus;
    float val = fade(dist * float(SAMPLE_COUNT) / line_width); // + fade(abs(sample_index - center_sample) * line_width / float(SAMPLE_COUNT));
    vec3 col = lch_srgb(vec3(0.8, 0.1, fract(pattern(UV * vec2(scale_x, 1.0), chrono) * 2.0) + chrono * 2.0));
    COLOR = vec4(val * col, 1.0);
}
