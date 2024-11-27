#version 450
#extension GL_GOOGLE_include_directive : require

#include "../define.glsl"
#include "../pc_3D.glsl"

layout(location = 0) in vec3 position;
layout(location = 1) lowp in vec3 normal;
layout(location = 2) lowp in uint nbtexture;
layout(location = 3) lowp in vec2 texcoord;
layout(location = 4) lowp in vec4 color;
layout(location = 5) lowp in uint color_blend_type;

layout(location = 0) lowp out vec3 v_normal;
layout(location = 1) lowp out vec4 v_color;
layout(location = 2) lowp out vec2 v_texcoord;
layout(location = 3) lowp out uint v_nbtexture;
layout(location = 4) lowp out uint v_color_blend_type;

void main() {
	//transpose(inverse(mat3(globals.worldview)))
    v_normal = normal;
    v_color = color;
    v_texcoord = texcoord;
    v_nbtexture = nbtexture;
	v_color_blend_type = color_blend_type;
    gl_Position = getGlobalsWorldViewProj() * vec4(position, 1.0);
}
