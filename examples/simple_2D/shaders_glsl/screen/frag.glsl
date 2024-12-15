#version 450
#extension GL_GOOGLE_include_directive : require

#include "../define.glsl"
#include "../utils_color.glsl"

layout(push_constant) uniform PushConstants {
    vec4 window;
    float time;
} globals;

// The `color_input` parameter of the `draw` method.
layout(input_attachment_index = 0, set = 1, binding = 0) uniform subpassInput render_one;
layout(input_attachment_index = 1, set = 1, binding = 1) uniform subpassInput render_two;

layout(location = 0) out vec4 f_color;

void main() {
    // Load the value at the current pixel.
    vec4 tmp = subpassLoad(render_one);
    vec4 tmp2 = subpassLoad(render_two);
    vec4 finalrgb = vec4(mix(tmp.rgb, tmp2.rgb, 1.0-tmp.a), 1.0);
    finalrgb.r = clamp(finalrgb.r, 0.0, 1.0);
    finalrgb.g = clamp(finalrgb.g, 0.0, 1.0);
    finalrgb.b = clamp(finalrgb.b, 0.0, 1.0);
    f_color = finalrgb;
}
