#version 300 es
// fragment shader
precision highp float;

in vec2 v_Uv;
in vec4 v_Color;

out vec4 o_Target;

uniform sampler2D TextureAtlas_texture;
// uniform sampler TextureAtlas_texture_sampler;

void main() {
    o_Target = v_Color * texture(
        TextureAtlas_texture,
        v_Uv);
}
