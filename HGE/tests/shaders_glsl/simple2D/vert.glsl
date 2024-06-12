#version 450
#extension GL_GOOGLE_include_directive : require

#include "../define.glsl"
#include "../pc_2D.glsl"

layout(location = 0) in vec3 position;
layout(location = 1) in uint ispixel;
layout(location = 2) in uint texture;
layout(location = 4) in vec2 uvcoord;
layout(location = 5) in vec4 color;
layout(location = 6) in uint color_blend_type;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec4 v_color;
layout(location = 2) out vec2 v_texcoord;
layout(location = 3) out uint v_nbtexture;
layout(location = 4) out uint v_color_blend_type;

void main() {
    v_normal = vec3(0.0,0.0,1.0);
    v_color = color;
    v_texcoord = uvcoord;
    v_nbtexture = texture;
	v_color_blend_type = color_blend_type;
    if(ispixel==1)
    {
        float posx = ((position.x / globals.window.x)*2.0) - 1.0;
        float posy = ((position.y / globals.window.y)*2.0) - 1.0;
        gl_Position = globals.world * vec4(posx,posy,position.z, 1.0);
    }
    else
    {
        gl_Position = globals.world * vec4(position, 1.0);
    }
}
