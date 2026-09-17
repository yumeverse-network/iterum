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

pub mod compute {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "assets/shaders/test.comp",
    }
}