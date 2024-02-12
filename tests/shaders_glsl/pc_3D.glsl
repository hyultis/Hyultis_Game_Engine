
#ifdef LOWPUSHCONSTANT

layout(push_constant) uniform PushConstants {
	mat4 projviewworld;
	vec2 window;
	float time;
} globals;

mat4 getGlobalsWorldViewProj()
{
	return globals.projviewworld;
}

#else

layout(push_constant) uniform PushConstants {
	mat4 world;
	mat4 view;
	mat4 proj;
	vec2 window;
	float time;
} globals;


mat4 getGlobalsWorldViewProj()
{
	return globals.proj * globals.view * globals.world;
}

#endif
