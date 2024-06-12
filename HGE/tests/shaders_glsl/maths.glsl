const float PI = 3.1415926535897932384626433832795;
const float RAD = 0.0174532925199432957692369076848;

mat3 rotation(float pitch, float yaw,  float roll)
{
	mat3 rotationMatrix;
	rotationMatrix[0] = vec3(
		cos(pitch)*cos(roll),
		cos(pitch)*sin(roll),
		-sin(pitch)
	);
	rotationMatrix[1] = vec3(
		sin(yaw)*sin(pitch)*cos(roll) - cos(yaw)*sin(roll),
		sin(yaw)*sin(pitch)*sin(roll) + cos(yaw)*cos(roll),
		sin(yaw)*cos(pitch)
	);
	rotationMatrix[2] = vec3(
		cos(yaw)*sin(pitch)*cos(roll) + sin(yaw)*sin(roll),
		cos(yaw)*sin(pitch)*sin(roll) - sin(yaw)*cos(roll),
		cos(yaw)*cos(pitch)
	);
	return rotationMatrix;
}
