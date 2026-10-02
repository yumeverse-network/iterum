#version 460

layout(location = 0) out vec2 v_ndc;

void main() {
    vec2 pos = vec2(float((gl_VertexIndex << 1) & 2), float(gl_VertexIndex & 2));
    v_ndc = pos * 2.0 - 1.0;
    gl_Position = vec4(v_ndc, 1.0, 1.0);
}