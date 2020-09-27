#version 300 es

precision highp float;

in vec2 v_Uv;

out vec4 o_Target;

layout(std140) uniform ColorMaterial_color {
    vec4 Color;
};

uniform sampler2D ColorMaterial_texture;

void main() {
    vec4 color = Color;
    color *= texture(
        ColorMaterial_texture,
        v_Uv
    );
    o_Target = color;
}
