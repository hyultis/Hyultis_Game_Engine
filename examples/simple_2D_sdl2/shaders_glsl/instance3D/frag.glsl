#version 450
#extension GL_GOOGLE_include_directive : require

#include "../define.glsl"
#include "../pc_3D.glsl"
#include "../textureSurclass.glsl"

layout(constant_id=0) const uint transparent = 0;

layout(location = 0) lowp in vec4 v_color;
layout(location = 1) lowp in vec2 v_texcoord;
layout(location = 2) lowp flat in uint v_nbtexture;

layout(location = 0) lowp out vec4 f_color;

void main()
{
	//float depthCalc = (gl_FragCoord.z / gl_FragCoord.w) / uniforms.window.z;
	lowp vec4 tmp_color = v_color;

	if (v_nbtexture>0)
	{
		lowp vec4 textureColor = getTextureSC(v_nbtexture, v_texcoord);
		tmp_color = textureColor * v_color;
	}

	if(transparent==0)
	{
		if (tmp_color.a<0.99)
			discard;
	}
	else
	{
		if(tmp_color.a>=0.99)
			discard;
	}

	f_color = tmp_color;
}
