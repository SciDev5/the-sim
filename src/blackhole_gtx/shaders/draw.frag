#version 450 core

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 FragColor;

layout (set = 0, binding = 0) uniform UniformBufferObject {
    vec2 view_dim;
    vec2 rotation;
    vec3 position;
    float fov_y;
    int activated;
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

///////// ----- COORDINATE UTILITIES ----- /////////

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

///////// ----- MATH UTILITIES ----- /////////

float sq(float x) {
    return x*x;
}

///////// ----- GR STUFF ----- /////////

const float RS = 1.0;
const float a = 0.5;

const mat4 g_minkowsky = mat4(
    -1,0,0,0,
    0,1,0,0,
    0,0,1,0,
    0,0,0,1
);

/// SCWARZSCHEILD

vec4 g_schwarzschild(vec4 x) {
    float r = x[1], th = x[2];
    float sin_th = sin(th);
    return vec4(
        -(1-RS/r),
        1/(1-RS/r),
        r*r,
        r*r*sin_th*sin_th
    );
}
vec4[4] dgdx_schwarzschild(vec4 x) {
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
mat4[4] christoffelsymbols_shwarzschild(vec4 x) {
    mat4[4] cs;

    vec4 g = g_schwarzschild(x);
    vec4[4] dgdx = dgdx_schwarzschild(x);

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
void step_geodesic_schwarzschild(float dt, inout vec4 x0, inout vec4 x1) {
    x0 += dt * x1 * 0.5;
    mat4[4] cs = christoffelsymbols_shwarzschild(x0);
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
void normalize_null_geodesic_schwarzschild(vec4 x0, inout vec4 x1) {
    vec4 g = g_schwarzschild(x0);

    x1.yzw = normalize(x1.yzw);
    x1[0] = sqrt(abs((dot(x1, g*x1) - g[0]*x1[0]*x1[0]) / (-g[0])));
}
bool trace_schwarzschild(inout vec4 x0, inout vec4 x1) {
    for (int i = 0; i < 1000; i++) {
        // float sin_th = sin(x0[2]);
        // float dt = min(0.01,-0.1/(1-2*x0[1])) * sin_th*sin_th;
        float dt = -0.025 * max(1.0, pow(0.1 * x0[1]/RS, 0.5) );
        normalize_null_geodesic_schwarzschild(x0, x1);
        step_geodesic_schwarzschild(dt, x0, x1);
        if (x0[1] < RS * 1.03) {
            return false;
        }
        if (x0[1] > 25*RS && x0[1]*-x1[1] > 0) {
            return true;
        }
    }
    return true;
}

//// KERR

mat4 g_kerr_ks(vec4 p) {
    float x = p[1], y = p[2], z = p[3];
    // float x = -sq(p[1]), y = -sq(p[2]), z = -sq(p[3]);

    float
        xx = x*x,
        yy = y*y,
        zz = z*z,
        RR = dot(p.yzw,p.yzw),
        aa = a * a;

    float r = sqrt(
        RR - aa + sqrt(
            RR*RR + aa*aa - 2*aa*(xx + yy - zz)
        )
    ) / sqrt(2), rr = r*r;
    
    // float f = RS * r*rr / (rr*rr + aa*zz);

    // vec4 k = vec4(
    //     1,
    //     (r * x + a * y) / (rr + aa),
    //     (r * y - a * x) / (rr + aa),
    //     z / r
    // );

    // return mat4(
        
    // );

    
    // // force a = 0
    // float r = length(vec3(x,y,z));
    // float f = RS / r;
    // vec4 k = vec4(0, vec3(x,y,z) / r);
    // // vec4 k = vec4(1, vec3(1,1,1));

    // return g_minkowsky - f * outerProduct(k, k);
    float A = 1-RS/r;

    // float k = sq(1/A);
    float k = 1/A;

    return mat4(
        - A, 0, 0, 0,
        0, k, 0 , 0,
        0, 0, k, 0,
        0, 0,0, k
    );
}
mat4[4] dgdx_kerr_ks(vec4 p) {
    const float DX = 1.52587891E-5;
    mat4[4] dgdx;
    dgdx[0] = mat4(
        vec4(0, 0, 0, 0),
        vec4(0, 0, 0, 0),
        vec4(0, 0, 0, 0),
        vec4(0, 0, 0, 0)
    );
    dgdx[1] = (g_kerr_ks(p + vec4(0,DX,0,0)) - g_kerr_ks(p - vec4(0,DX,0,0))) / (2 * DX);
    dgdx[2] = (g_kerr_ks(p + vec4(0,0,DX,0)) - g_kerr_ks(p - vec4(0,0,DX,0))) / (2 * DX);
    dgdx[3] = (g_kerr_ks(p + vec4(0,0,0,DX)) - g_kerr_ks(p - vec4(0,0,0,DX))) / (2 * DX);
    return dgdx;
}
mat4[4] christoffelsymbols_kerr_ks(vec4 x) {
    mat4[4] cs;

    mat4 g = g_kerr_ks(x);
    mat4 g_inv = inverse(g);
    mat4[4] dgdx = dgdx_kerr_ks(x);

    for (int l = 0; l < 4; l++) {
        mat4 m = mat4(
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0
        );
        for (int k = 0; k < 4; k++) {
            mat4 m_inner = mat4(
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0
            );
            for (int i = 0; i < 4; i++) {
                for (int j = 0; j < 4; j++) {
                    // + d(i) g[j,k]
                    // m_inner[i][k] += dgdx[i] [j][k];
                    m_inner[i][k] += dgdx[i] [k][j];
                    // + d(j) g[i,k]
                    // m_inner[k][j] += dgdx[j] [i][k];
                    m_inner[k][j] += dgdx[j] [k][i];
                    // - d(k) g[i,j]
                    m_inner[i][j] -= dgdx[k] [i][j]; // correct
                }
            }

            m += m_inner / 2 * g_inv[k][l];
        }
        cs[l] = m;
    }

    return cs;
}
void step_geodesic_kerr_ks(float dt, inout vec4 x0, inout vec4 x1) {
    x0 += dt * x1 * 0.5;
    mat4[4] cs = christoffelsymbols_kerr_ks(x0);
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

void normalize_null_geodesic_kerr_ks(vec4 x0, inout vec4 x1) {
    mat4 g = g_kerr_ks(x0);

    // x1.yzw = normalize(x1.yzw);
    float A = g[0][0];
    float B = 2 * dot(g[0].yzw, x1.yzw);
    float C = dot(mat3(g[1].yzw,g[2].yzw,g[3].yzw) * x1.yzw, x1.yzw);
    x1[0] = (-B - sqrt(B*B - 4*A*C))/(2*A); // -sqrt in quadratic formula selects for the time-reversed version.
}

bool trace_kerr_ks(inout vec4 x0, inout vec4 x1) {
    for (int i = 0; i < 1000; i++) {
        float dt = 0.05;
        normalize_null_geodesic_kerr_ks(x0, x1);
        step_geodesic_kerr_ks(dt, x0, x1);

        float r = length(x0.yzw);
        if (r < RS) {
            return false;
        }
        // if (r > 25*RS) {
        //     return true;
        // }
    }
    return true;
}

void main() {
    vec3 d = init_raydir();
    vec3 p = camera.position;

    if (camera.activated != 0
    // ||true
    ) {
        // // Schwarzschild
        // vec4 x0 = vec4(0, rectilinear_to_spherical(p));
        // vec4 x1 = -vec4(0, tangent_rectilinear_to_spherical_mat(x0.yzw) * d);
        // if (trace_schwarzschild(x0, x1)) {
        //     FragColor = background(tangent_spherical_to_rectilinear_mat(x0.yzw) * -x1.yzw);
        // } else {
        //     FragColor = vec4(0,0,0,0);
        // }

        // Kerr (Kerr-Schild coordinates)
        vec4 x0 = vec4(0, p);
        vec4 x1 = -vec4(0, d);
        if (trace_kerr_ks(x0, x1)) {
            FragColor = background(-x1.yzw);
        } else {
            FragColor = vec4(0,0,0,0);
        }

        // vec2 v = vec2(uv.x * camera.view_dim.x / camera.view_dim.y, -uv.y);
        // vec4 x0 = vec4(0,v.x,0,v.y)*5 + vec4(0,p);
        // // vec4 x0 = vec4(0,v.x,v.y,0.0)*5 + vec4(0,p);
        // // mat4 g = g_kerr_ks(x0);
        // // for (int i = 0; i < 4; i++) {
        // //     for (int j = 0; j < 4; j++) {
        // //         float k = g[i][j];
        // //         FragColor += vec4(k/5,-k/5,mod(k,1),0) / 16;
        // //     }
        // // }
        // // vec4 g = g_schwarzschild(x0);
        // // for (int i = 0; i < 4; i++) {
        // //     float k = g[i];
        // //     FragColor += vec4(k/5,-k/5,mod(k,1),0) / 4;
        // // }
        // vec4 x1 = vec4(0,normalize(x0.yzw));
        // normalize_null_geodesic_kerr_ks(x0, x1);
        // // normalize_null_geodesic_schwarzschild(x0, x1);
        // // float ds = dot(x1,x1*g); // should be 0 now
        // // float k = ds;

        
        // mat4[4] cs = christoffelsymbols_kerr_ks(x0);
        // // mat4[4] cs = christoffelsymbols_shwarzschild(x0);
        // vec4 x2 = -vec4(
        //     dot(cs[0]*x1, x1),
        //     dot(cs[1]*x1, x1),
        //     dot(cs[2]*x1, x1),
        //     dot(cs[3]*x1, x1)
        // );
        // float k = x2[0];

        // FragColor = vec4(-k/2,0*mod(k,1),k/2,0);
        
        // // mat4[4] g = christoffelsymbols_kerr_ks(vec4(0,v.x,0,v.y)*5);
        // // for (int i = 0; i < 4; i++) {
        // //     for (int j = 0; j < 4; j++) {
        // //         float k = g[0][i][j];
        // //         FragColor += vec4(k/5,-k/5,mod(k,1),0) / 16;
        // //     }
        // // }
        
    } else {
        FragColor = background(d);
        if (acos(dot(-d,p)) < atan(1/length(p))) {
            FragColor = vec4(0,0,0.5,0);
        }
    }




    // if (acos(dot(normalize(p),d)) < atan(0.1, length(p))) {
    //     FragColor = vec4(0,0,0,0);
    // }
}
