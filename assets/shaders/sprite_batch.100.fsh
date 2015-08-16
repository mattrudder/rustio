#version 100
uniform sampler2D tex;
varying lowp vec2 v_tex_coords;
varying lowp vec4 v_color;
flat varying uint v_tex_id;
void main() {
    gl_FragColor = texture(tex, v_tex_coords);
    //gl_FragColor = v_color;
}
