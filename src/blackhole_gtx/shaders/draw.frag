#version 450 core

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 FragColor;

layout (set = 0, binding = 0) uniform UniformBufferObject {
    vec2 view_dim;
    vec2 rotation;
    vec3 position;
    float fov_y;
} camera;

vec4 background(vec3 dir) {
    vec3 d = normalize(dir);
    float grid = 0;
    if ((mod(d.x, 0.1) > 0.05 == mod(d.y, 0.1) > 0.05) == mod(d.z, 0.1) > 0.05) {
        grid = 1;
    }
    return vec4(
        max(0, d.x) * (grid * 0.4 + 0.6) - grid * 0.1 * d.x,
        max(0, d.y) * (grid * 0.4 + 0.6) - grid * 0.1 * d.y,
        max(0, d.z) * (grid * 0.4 + 0.6) - grid * 0.1 * d.z,
        1.0
    );
}

mat3 rotateX(float t) {
    float s = sin(t);
    float c = cos(t);
    return mat3(
        1, 0, 0,
        0, c, -s,
        0, s, c
    );
}
mat3 rotateY(float t) {
    float s = sin(t);
    float c = cos(t);
    return mat3(
        c, 0, s,
        0, 1, 0,
        -s, 0, c
    );
}
mat3 rotateZ(float t) {
    float s = sin(t);
    float c = cos(t);
    return mat3(
        c, -s, 0,
        s, c, 0,
        0, 0, 1
    );
}

vec3 init_raydir() {
    vec3 p = normalize(vec3(
        uv.x * camera.view_dim.x / camera.view_dim.y,
        -uv.y,
        1 / tan(camera.fov_y / 2)
    ));
    p = rotateX(camera.rotation[1]) * p; // pitch
    p = rotateZ(camera.rotation[0]) * p; // yaw
    return p;
}

// Rectilinear -> (x,y,z)
// Spherical -> (r,th,ph) ; [
//     th=0    @ ( 0,         0,         r  )
//     th=PI/2 @ ( r*cos(ph), r*sin(th), 0  )
//     th=PI   @ ( 0,         0,         -r )
// ]

vec3 spherical_to_rectilinear(vec3 v) {
    float r = v.x, th = v.y, ph = v.z;
    return r * vec3(
        sin(th) * cos(ph),
        sin(th) * sin(ph),
        cos(th)
    );
}
vec3 rectilinear_to_spherical(vec3 v) {
    float r = length(v);
    if (r == 0) {
        return vec3(0,0,0);
    }
    vec3 vn = normalize(v);
    if (vn.y == 0 && vn.x == 0) {
        return vec3(
            r,
            acos(vn.z),
            0
        );
    } else {
        return vec3(
            r,
            acos(vn.z),
            atan(vn.y, vn.x)
        );
    }
}

void main() {
    vec3 d = init_raydir();
    vec3 p = camera.position;

    vec3 d_sph = rectilinear_to_spherical(d);
    vec3 d_rec = spherical_to_rectilinear(d_sph);

    FragColor = abs(background(d) - background(d_rec));
    // FragColor = background(d);

    if (d.z > 0.9) {
        FragColor = vec4(0,0,0,0);
    }
}
