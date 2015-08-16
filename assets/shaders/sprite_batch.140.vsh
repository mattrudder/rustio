#version 140
in vec2 position;
in vec2 uv;
in vec4 color;
in uint tex_id;
out vec2 v_tex_coords;
out vec4 v_color;
flat out uint v_tex_id;
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
    //v_tex_id = tex_id;
    v_tex_coords = uv;
    v_color = color;
}
