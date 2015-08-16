#version 100
attribute lowp vec2 position;
attribute lowp vec2 uv;
attribute lowp vec4 color;
attribute uint tex_id;
varying lowp vec2 v_tex_coords;
varying lowp vec4 v_color;
flat varying uint v_tex_id;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    // if (gl_VertexID % 4 == 0) {
    //     v_tex_coords = vec2(0.0, 1.0);
    // } else if (gl_VertexID % 4 == 1) {
    //     v_tex_coords = vec2(1.0, 1.0);
    // } else if (gl_VertexID % 4 == 2) {
    //     v_tex_coords = vec2(0.0, 0.0);
    // } else {
    //     v_tex_coords = vec2(1.0, 0.0);
    // }
    // v_tex_id = i_tex_id;
    v_tex_coords = uv;
    v_color = color;
}
