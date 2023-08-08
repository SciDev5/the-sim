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

mat3 tangent_rectilinear_to_spherical_mat(vec3 p_sph) {
    vec3 dr = -spherical_to_rectilinear(vec3(1, p_sph.yz));
    vec3 dth = -spherical_to_rectilinear(vec3(1 / p_sph.x, p_sph.y + 3.14159/2, p_sph.z));
    vec3 dph = -spherical_to_rectilinear(vec3(1 / (p_sph.x * sin(p_sph.y)), 3.14159/2, p_sph.z + 3.14159/2));

    return mat3(
        dr.x, dth.x, dph.x,
        dr.y, dth.y, dph.y,
        dr.z, dth.z, dph.z
    );
}

mat3 tangent_spherical_to_rectilinear_mat(vec3 p_sph) {
    return inverse(tangent_rectilinear_to_spherical_mat(p_sph));
}

const float RS = 1.0;

vec4 g_diag(vec4 x) {
    float r = x[1], th = x[2];
    float sin_th = sin(th);
    return vec4(
        -(1-RS/r),
        1/(1-RS/r),
        r*r,
        r*r*sin_th*sin_th
    );
}
vec4[4] dgdx_diag(vec4 x) {
    float r = x[1], th = x[2];
    float sin_th = sin(th);
    float cos_th = cos(th);
    vec4[4] dgdx;
    dgdx[0] = vec4(0,0,0,0); // dg/dt
    dgdx[1] = vec4(-RS/(r*r),-RS/((RS-r)*(RS-r)),2*r,2*r*sin_th*sin_th); // dg/dr
    dgdx[2] = vec4(0,0,0,2*r*r*sin_th*cos_th); // dg/d<th>
    dgdx[3] = vec4(0,0,0,0); // dg/d<ph>
    return dgdx;
}

mat4[4] christoffelsymbols_gdiag(vec4 x) {
    mat4[4] cs;

    vec4 g = g_diag(x);
    vec4[4] dgdx = dgdx_diag(x);

    for (int k = 0; k < 4; k++) {
        mat4 m = mat4(0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0);
        // + d(i) g[j,k] :: j = k
        // + d(j) g[i,k] :: i = k
        for (int i = 0; i < 4; i++) {
            m[i][k] += dgdx[i][k];
            m[k][i] += dgdx[i][k];
        }
        // - d(k) g[i,j] :: i = j
        for (int i = 0; i < 4; i++) {
            m[i][i] -= dgdx[k][i];
        }
        cs[k] = m / (2 * g[k]);
    }

    return cs;
}

void step_geodesic_gdiag(float dt, inout vec4 x0, inout vec4 x1) {
    x0 += dt * x1 * 0.5;
    mat4[4] cs = christoffelsymbols_gdiag(x0);
    vec4 x2 = -vec4(
        dot(cs[0]*x1, x1),
        dot(cs[1]*x1, x1),
        dot(cs[2]*x1, x1),
        dot(cs[3]*x1, x1)
    );
    x1 += dt * x2;
    x0 += dt * x1 * 0.5;
    // x0 += dt * x1;
}

void normalize_null_geodesic_gdiag(vec4 x0, inout vec4 x1) {
    vec4 g = g_diag(x0);

    x1.yzw = normalize(x1.yzw);
    x1[0] = sqrt(abs((dot(x1, g*x1) - g[0]*x1[0]*x1[0]) / (-g[0])));
}

bool trace_gdiag(inout vec4 x0, inout vec4 x1) {
    for (int i = 0; i < 1000; i++) {
        // float sin_th = sin(x0[2]);
        // float dt = min(0.01,-0.1/(1-2*x0[1])) * sin_th*sin_th;
        float dt = -0.025 * max(1.0, pow(0.1 * x0[1]/RS, 0.5) );
        normalize_null_geodesic_gdiag(x0, x1);
        step_geodesic_gdiag(dt, x0, x1);
        if (x0[1] < RS * 1.03) {
            return false;
        }
        if (x0[1] > 25*RS && x0[1]*-x1[1] > 0) {
            break;
        }
    }
    return true;
}

void main() {
    vec3 d = init_raydir();
    vec3 p = camera.position;

    vec4 x0 = vec4(0, rectilinear_to_spherical(p));
    vec4 x1 = -vec4(0, tangent_rectilinear_to_spherical_mat(x0.yzw) * d);
    
    if (trace_gdiag(x0, x1)) {
        FragColor = background(tangent_spherical_to_rectilinear_mat(x0.yzw) * -x1.yzw);
    } else {
        FragColor = vec4(0,0,0,0);
    }


    // if (acos(dot(normalize(p),d)) < atan(0.1, length(p))) {
    //     FragColor = vec4(0,0,0,0);
    // }
}
