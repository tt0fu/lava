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
    lsrgb = clamp(lsrgb, 0.0, 1.0);
    vec3 xlo = 12.92 * lsrgb;
    vec3 xhi = 1.055 * pow(lsrgb, vec3(0.4166666666666667)) - 0.055;
    return mix(xlo, xhi, step(vec3(0.0031308), lsrgb));
}

vec3 srgb_lsrgb(vec3 srgb) {
    srgb = clamp(srgb, 0.0, 1.0);
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
