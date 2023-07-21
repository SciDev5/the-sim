#version 450 core

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 uv;

void main() {
    uv = vec2(position.x, 1 - position.y);
    gl_Position = vec4(position * 2 - 1, 0, 1);
}
