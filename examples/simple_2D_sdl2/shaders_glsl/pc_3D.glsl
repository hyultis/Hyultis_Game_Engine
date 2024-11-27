
layout(push_constant) uniform PushConstants {
	mat4 projviewworld;
	vec2 window;
	float time;
} globals;

mat4 getGlobalsWorldViewProj()
{
	return globals.projviewworld;
}
