#version 450 core

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 FragColor;

//INJECT// layout(set = 0, binding = 0) uniform texture2D cattex_texture;
//INJECT// layout(set = 0, binding = 1) uniform sampler cattex_sampler;
vec4 sample_cattex(vec2 coord) {
    return vec4(0, 0, 0, 0); //REPLACE// return texture(sampler2D(cattex_texture, cattex_sampler), coord);
}
//INJECT// layout(set = 1, binding = 0) uniform texture2D bufftex_texture;
//INJECT// layout(set = 1, binding = 1) uniform sampler bufftex_sampler;
vec4 sample_bufftex(vec2 coord) {
    return vec4(0, 0, 0, 0); //REPLACE// return texture(sampler2D(bufftex_texture, bufftex_sampler), coord);
}

void main() {
    FragColor = vec4(uv, 1.0, 1.0) * sample_cattex(uv) * sample_bufftex(uv);
}
