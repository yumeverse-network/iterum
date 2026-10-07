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