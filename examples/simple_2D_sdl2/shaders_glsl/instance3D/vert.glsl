#version 450
#extension GL_GOOGLE_include_directive : require

#include "../define.glsl"
#include "../pc_3D.glsl"
#include "../maths.glsl"
#include "../utils_color.glsl"

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texcoord;
layout(location = 2) in vec4 color;
layout(location = 3) in vec3 instance_offset;
layout(location = 4) in vec3 instance_scale;
layout(location = 5) in vec3 instance_rotation;
layout(location = 6) in vec4 instance_color;
layout(location = 7) in uint instance_texture;
layout(location = 8) in vec2 instance_texcoord_offset;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_texcoord;
layout(location = 2) out uint v_nbtexture;

void main()
{
    gl_Position = getGlobalsWorldViewProj() * vec4(position * rotation(instance_rotation.x*RAD, instance_rotation.y*RAD, instance_rotation.z*RAD) * instance_scale + instance_offset, 1.0);

    v_texcoord = texcoord+instance_texcoord_offset;
    v_color = instance_color*color;
    v_nbtexture = instance_texture;
}
