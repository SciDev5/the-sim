#version 450 core

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 uv;

void main() {
    uv = vec2(0.5 + 0.5 * position.x, 0.5 - 0.5 * position.y);
    gl_Position = vec4(position, 0, 1);
}
