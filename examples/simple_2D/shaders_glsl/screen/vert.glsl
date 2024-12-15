#version 450

layout(push_constant) uniform PushConstants {
    vec4 window;
    float time;
} uniforms;

layout(location = 0) in vec2 position;

void main()
{
    gl_Position = vec4(position, 0.0, 1.0);
}
