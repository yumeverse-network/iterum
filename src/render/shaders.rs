// Main lighting. Sorry about the name of triangle
pub mod vertex {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "assets/shaders/triangle.vert",
    }
}
pub mod fragment {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "assets/shaders/triangle.frag",
    }
}

// Sky
pub mod sky_vertex {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "assets/shaders/sky.vert",
    }
}
pub mod sky_fragment {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "assets/shaders/sky.frag",
    }
}

// Literally nothing
pub mod compute {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "assets/shaders/test.comp",
    }
}

// UI
pub mod ui_vertex {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r#"
            #version 450
            layout(location = 0) in vec2 pos;
            layout(location = 1) in vec2 uv;
            layout(location = 2) in vec4 col;
            layout(push_constant) uniform PC { vec2 scale; vec2 translate; } pc;
            layout(location = 0) out vec2 v_uv;
            layout(location = 1) out vec4 v_col;
            void main() {
                v_uv = uv;
                v_col = col;
                gl_Position = vec4(pos * pc.scale + pc.translate, 0.0, 1.0);
            }
        "#
    }
}
pub mod ui_fragment {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r#"
            #version 450
            layout(location = 0) in vec2 v_uv;
            layout(location = 1) in vec4 v_col;
            layout(set = 0, binding = 0) uniform sampler2D tex;
            layout(location = 0) out vec4 out_color;
            void main() {
                out_color = v_col * texture(tex, v_uv);
            }
        "#
    }
}

// UI Selection Outline
pub mod outline_vertex {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "assets/shaders/outline.vert",
    }
}

pub mod outline_fragment {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "assets/shaders/outline.frag",
    }
}