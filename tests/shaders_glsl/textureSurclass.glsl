
#ifndef VULKAN11COMP
#extension GL_EXT_nonuniform_qualifier : require

layout(set = 1, binding = 0) uniform sampler2D thetexture[];

vec4 getTextureSC(uint idTexture, vec2 uvcoord)
{
	return texture(nonuniformEXT(thetexture[idTexture]), uvcoord);
}

#else

layout(set = 0, binding = 0) uniform sampler2D font;
layout(set = 1, binding = 0) uniform sampler2DArray texture_small;
layout(set = 2, binding = 0) uniform sampler2DArray texture_large;

vec4 getTextureSC(uint idTexture, vec2 uvcoord)
{
	if(idTexture>=9999)
	{
		return texture(font, uvcoord);
	}
	if(idTexture>=200)
	{
		return texture(texture_large, vec3(uvcoord,idTexture-200));
	}
	return texture(texture_small, vec3(uvcoord,idTexture));
}

#endif
