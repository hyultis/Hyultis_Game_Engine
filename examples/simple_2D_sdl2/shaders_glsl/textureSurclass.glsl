
layout(set = 0, binding = 0) uniform sampler2D font;
layout(set = 1, binding = 0) uniform sampler2DArray texture_small;
layout(set = 2, binding = 0) uniform sampler2DArray texture_large;

uint[2] getChannelTextureId(uint packed) // packed as LE u32
{
	uint channel = (packed >> 24u) & 0xFFu;
	uint texture = packed & 0xFFFFFFu;
	return uint[2](channel,texture);
}

vec4 getTextureSC(uint idTexture, vec2 uvcoord)
{
	if(idTexture==0)
	{
		return vec4(0.0,0.0,0.0,0.0);
	}

	uint[2] unpacked = getChannelTextureId(idTexture-1); // 0 is no texture
	if(unpacked[0]==0)
	{
		return texture(font, uvcoord);
	}
	if(unpacked[0]==2)
	{
		return texture(texture_large, vec3(uvcoord,unpacked[1]));
	}
	return texture(texture_small, vec3(uvcoord,unpacked[1]));
}
