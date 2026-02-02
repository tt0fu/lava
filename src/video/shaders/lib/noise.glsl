#ifndef NOISE
#define NOISE

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

float fbm3(vec2 p, float time) {
    return fbm(p + fbm(p + fbm(p, time), time), time);
}

#endif