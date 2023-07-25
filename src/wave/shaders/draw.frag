#version 450 core

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 FragColor;

//INJECT// layout(set = 0, binding = 0) uniform texture2D tex0_tex;
//INJECT// layout(set = 0, binding = 1) uniform sampler tex0_samp;
vec4 sample_bufftex(vec2 coord) {
    return vec4(0, 0, 0, 0); //REPLACE// return texture(sampler2D(tex0_tex, tex0_samp), coord);
}

void main() {
    FragColor = sample_bufftex(uv);// + vec4(0,0,1,0);
}
