#version 450 core

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 FragColor;

layout (set = 0, binding = 0) uniform UniformBufferObject {
    vec2 view_dim;
    vec2 rotation;
    vec3 position;
    float fov_y;
    int activated;
    float a;
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

const float RS = 1.0; // Placeholder, must be 1.0

mat4 g_kerr_bl(vec4 p) {
    float
        a = camera.a,
        r = p[1],
        th = p[2],
        cos_th = cos(th),
        sin_th = sin(th),
        ph = p[3],
        sigma = r*r + a*a*cos_th*cos_th,
        delta = r*r-RS*r+a*a;
    
    const float C = 1;

    float
        t_t = -(1-RS*r/sigma)*C*C,
        r_r = sigma/delta,
        th_th = sigma,
        ph_ph = (r*r+a*a+RS*r*a*a/sigma*sin_th*sin_th)*sin_th*sin_th,
        t_ph = -2*C*RS*r*a*sin_th*sin_th/sigma;

    return mat4(
      t_t, 0, 0, t_ph,
      0, r_r, 0, 0,
      0, 0, th_th, 0,
      t_ph, 0, 0, ph_ph 
    );
}
void normalize_null_geodesic_kerr_bl(vec4 x0, inout vec4 x1) {
    mat4 g = g_kerr_bl(x0);

    x1.yzw = normalize(x1.yzw);
    float A = g[0][0];
    float B = 2 * dot(g[0].yzw, x1.yzw);
    float C = dot(mat3(g[1].yzw,g[2].yzw,g[3].yzw) * x1.yzw, x1.yzw);
    x1[0] = (-B - sqrt(B*B - 4*A*C))/(2*A); // -sqrt in quadratic formula selects for the time-reversed version.
}

bool trace_kerr_bl(inout vec4 x0, inout vec4 x1) {
    float
        p_t = x1[0],
        p_r = x1[1],
        p_th = x1[2],
        p_ph = x1[3],
        t = x0[0],
        r = x0[1],
        th = x0[2],
        ph = x0[3],
        a = camera.a,
        delta = r * r - r + a * a,
        sigma = r * r + sq(a * cos(th));

    float E = -p_t;
    float L = p_th;
    float Q = sq(p_th) + sq(cos(th)) * (
        -sq(a * E) + sq(L/sin(th))
    );


    for (int i = 0; i < 1; i++) {
        const float DT = 0.05;

        delta = r * r - r + a * a;
        sigma = r * r + sq(a * cos(th));

        float TH = Q - sq(cos(th)) * (
            -sq(a*E) + sq(L/sin(th))
        );
        float P = E*(sq(r)+sq(a)) - a*L;
        float R = sq(P) - delta * (
            sq(L-a*E) + Q
        );

        float k_t = (-a * (a*E*sq(sin(th))-L)+(r*r+a*a)*P/delta) / sigma;
        float k_r = (-sqrt(R)) / sigma;
        float k_th = (sqrt(TH)) / sigma;
        float k_ph = (-(a*E-L/sq(sin(th))) + a*P/delta) / sigma; // perhaps L is supposed to be squared?

        // p_t = k_t;
        // p_r = k_r;
        // p_th = k_th;
        // p_ph = k_ph;

        t += p_t * DT;
        r += p_r * DT;
        th += p_th * DT;
        ph += p_ph * DT;

        if (
            isnan(t) ||
            isnan(r) ||
            isnan(ph) ||
            isnan(th)
        ) {
            return false;
        }
    }

    x0 = vec4(t, r, th, ph);
    x1 = vec4(p_t, p_r, p_th, p_ph);

    return true;
}


void main() {
    vec3 d = init_raydir();
    vec3 p = camera.position;

    if (camera.activated == 0
    // ||true
    ) {
        
        vec4 x0 = vec4(0, rectilinear_to_spherical(p));
        vec4 x1 = vec4(0, tangent_rectilinear_to_spherical_mat(x0.yzw) * d);

        normalize_null_geodesic_kerr_bl(x0, x1);
        
        if (trace_kerr_bl(x0, x1)) {
            FragColor = background(tangent_spherical_to_rectilinear_mat(x0.yzw) * x1.yzw);
        } else {
            FragColor = vec4(1,0,1,0);
        }

    } else {
        FragColor = background(d);
        if (acos(dot(-d,normalize(p))) < atan(1/length(p))) {
            FragColor = vec4(0,0,0.5,0);
        }
    }




    // if (acos(dot(normalize(p),d)) < atan(0.1, length(p))) {
    //     FragColor = vec4(0,0,0,0);
    // }
}
