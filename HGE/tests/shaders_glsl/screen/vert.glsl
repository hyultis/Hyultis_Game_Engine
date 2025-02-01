#version 450

layout(push_constant) uniform PushConstants {
    vec4 window;
    float time;
} globals;

layout(location = 0) in vec2 position;

void main()
{
    float time = globals.time;// need globals to be used or its removed by something and throw a VUID-vkCmdPushConstants-offset-01795 error
    gl_Position = vec4(position, 0.0, 1.0);
}
