#version 450
#extension GL_GOOGLE_include_directive : require

#include "../define.glsl"
#include "../pc_3D.glsl"
#include "../textureSurclass.glsl"

layout(constant_id=0) const uint transparent = 0;

layout(location = 0) lowp in vec3 v_normal;
layout(location = 1) lowp in vec4 v_color;
layout(location = 2) lowp in vec2 v_texcoord;
layout(location = 3) lowp flat in uint v_nbtexture;
layout(location = 4) lowp flat in uint v_color_blend_type;

layout(location = 0) lowp out vec4 f_color;


void main()
{
	//float depthCalc = (gl_FragCoord.z / gl_FragCoord.w) / uniforms.window.z;
	vec4 tmp_color = vec4(0.0, 0.0, 0.0, 1.0);

	if (v_nbtexture==0)
	{
		tmp_color = v_color;
	}
	else
	{
		vec4 textureColor = getTextureSC(v_nbtexture-1, v_texcoord);
		if(v_color_blend_type==0)
		{
			tmp_color = textureColor*v_color;
		}
		else if(v_color_blend_type==1)
		{
			tmp_color = textureColor+v_color;
		}
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
