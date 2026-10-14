use crate::engine::Engine;
use crate::engine::fps::FpsCounter;
use crate::engine::player::Player;

pub struct EditorUi {
    pub show_demo: bool,
    pub show_right: bool,
    pub show_bottom: bool,
    pub show_left: bool,
    pub show_viewport: bool,
    pub viewport_size: [u32; 2],
    pub viewport_tex_id: Option<imgui::TextureId>,
    pub hovering_viewport: bool,
    pub camera_fov: f32,
    pub camera_near: f32,
    pub camera_far: f32,
    pub move_speed: f32,
    pub fast_speed: f32,
}

impl EditorUi {
    pub fn new() -> Self {
        Self {
            show_demo: false,
            show_right: true,
            show_bottom: true,
            show_left: true,
            show_viewport: true,
            hovering_viewport: false,
            viewport_size: [1280, 720],
            viewport_tex_id: None,
            camera_fov: 70.0,
            camera_near: 0.1,
            camera_far: 1.0e12,
            move_speed: 10.0,
            fast_speed: 2.0e10,
        }
    }

    pub fn draw(
        &mut self,
        ui: &imgui::Ui,
        engine: &mut Engine,
        player: &mut Player,
        fps: &FpsCounter,
    ) {
        let display_size = ui.io().display_size;
        let menu_bar_height = ui.frame_height();
        
        let mut ui_flags = imgui::WindowFlags::NO_MOVE;
        ui_flags |= imgui::WindowFlags::NO_RESIZE;
        ui_flags |= imgui::WindowFlags::NO_COLLAPSE;
        ui_flags |= imgui::WindowFlags::NO_TITLE_BAR;
        ui_flags |= imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS;
        ui_flags |= imgui::WindowFlags::NO_SAVED_SETTINGS;
        
        if self.show_demo {
            ui.show_demo_window(&mut self.show_demo);
        }

        if self.show_viewport {
            let size = [display_size[0] * 0.6, display_size[1] * 0.7];
            let position = [display_size[0] * 0.2, menu_bar_height];

            ui.window("Viewport")
                .position(position, imgui::Condition::Always)
                .size(size, imgui::Condition::Always)
                .opened(&mut self.show_viewport)
                .flags(ui_flags)
                .build(|| {
                    self.hovering_viewport = false;
                    
                    if let Some(_tab_bar) = ui.tab_bar("Tabs") {
                        if let Some(_tab1) = ui.tab_item("Scene") {
                            if let Some(tex) = self.viewport_tex_id {
                                let avail = ui.content_region_avail();
                                self.viewport_size = [
                                    avail[0].max(1.0) as u32,
                                    avail[1].max(1.0) as u32,
                                ];
                                imgui::Image::new(tex, avail).build(ui);

                                self.hovering_viewport = ui.is_window_hovered();
                            }
                        }
                        if let Some(_tab2) = ui.tab_item("Game") {
                            ui.text("Camera inactive.");
                        }
                    }
                });
        }

        if self.show_right {
            let size = [display_size[0] * 0.2, display_size[1]];
            let position = [display_size[0] - size[0], menu_bar_height];
            
            ui.window("Right")
                .position(position, imgui::Condition::Always)
                .size(size, imgui::Condition::Always)
                .opened(&mut self.show_right)
                .flags(ui_flags)
                .build(|| {
                    if let Some(_tab_bar) = ui.tab_bar("Tabs") {
                        if let Some(_tab1) = ui.tab_item("Inspector") {
                            if ui
                                .slider_config("FOV", 30.0, 120.0)
                                .build(&mut self.camera_fov)
                            {
                                engine.camera.fov = self.camera_fov;
                            }
                            if ui
                                .slider_config("Near", 0.001, 10.0)
                                .build(&mut self.camera_near)
                            {
                                engine.camera.near = self.camera_near;
                            }
                            if ui
                                .slider_config("Far", 100.0, 1.0e12)
                                .build(&mut self.camera_far)
                            {
                                engine.camera.far = self.camera_far;
                            }
                        }
                        if let Some(_tab2) = ui.tab_item("Something") {
                            ui.text("Nothing");
                        }
                    }
                });
        }

        if self.show_left {
            let size = [display_size[0] * 0.2, display_size[1] * 0.7];
            let position = [0.0, menu_bar_height];
            
            ui.window("Left")
                .position(position, imgui::Condition::Always)
                .size(size, imgui::Condition::Always)
                .opened(&mut self.show_left)
                .flags(ui_flags)
                .build(|| {
                    if let Some(_tab_bar) = ui.tab_bar("Tabs") {
                        if let Some(_tab1) = ui.tab_item("Hierarchy") {
                            let mut p = [
                                player.position.x as f32,
                                player.position.y as f32,
                                player.position.z as f32,
                            ];
                            if ui.input_float3("Position", &mut p).build() {
                                player.position.x = p[0] as f64;
                                player.position.y = p[1] as f64;
                                player.position.z = p[2] as f64;
                            }
                            ui.slider_config("Speed", 1.0, 1.0e6)
                                .build(&mut self.move_speed);
                            ui.slider_config("Fast speed", 1.0, 1.0e12)
                                .build(&mut self.fast_speed);
                            if ui.button("Reset") {
                                player.position = glam::DVec3::ZERO;
                            }
                        }
                        if let Some(_tab2) = ui.tab_item("Unknown") {
                            ui.text("Tab 2");
                        }
                    }
                });
        }

        if self.show_bottom {
            let size = [display_size[0] * 0.8, display_size[1] * 0.3];
            let position = [0.0, menu_bar_height + (display_size[1] * 0.7)];

            ui.window("Bottom")
                .position(position, imgui::Condition::Always)
                .size(size, imgui::Condition::Always)
                .opened(&mut self.show_left)
                .flags(ui_flags)
                .build(|| {
                    if let Some(_tab_bar) = ui.tab_bar("Tabs") {
                        if let Some(_tab1) = ui.tab_item("Assets") {
                            ui.text("Nothing here.");
                        }
                        if let Some(_tab2) = ui.tab_item("Console") {
                            ui.text("Console should appear later.");
                        }
                        if let Some(_tab3) = ui.tab_item("Performance") {
                            ui.text(format!("FPS:  {:.1}", fps.get_fps()));
                            ui.text(format!("dt:   {:.3} ms", engine.get_delta() * 1000.0));
                            ui.text(format!(
                                "Pos:  ({:.2}, {:.2}, {:.2})",
                                player.position.x, player.position.y, player.position.z
                            ));
                        }
                    }
                });
        }
            
        if let Some(_bar) = ui.begin_main_menu_bar() {
            if let Some(_menu) = ui.begin_menu("File") {
                ui.menu_item("New Project");
                ui.menu_item("Open Project");

                ui.separator();
                
                ui.menu_item("Save");
                ui.menu_item("Save As...");
                
                ui.separator();
                
                ui.menu_item("Close Project");
                ui.menu_item("Exit");
            }
            if let Some(_menu) = ui.begin_menu("Edit") {
                ui.menu_item("Undo");
                ui.menu_item("Redo");
                ui.menu_item("Copy");
                ui.menu_item("Paste");
                ui.menu_item("Cut");
                ui.menu_item("Settings");
            }
            if let Some(_menu) = ui.begin_menu("Help") {
                ui.menu_item("Docs");
            }
            if let Some(_menu) = ui.begin_menu("Test") {
                ui.menu_item_config("Right")
                    .selected(self.show_right)
                    .build()
                    .then(|| self.show_right = !self.show_right);
                ui.menu_item_config("Bottom")
                    .selected(self.show_bottom)
                    .build()
                    .then(|| self.show_bottom = !self.show_bottom);
                ui.menu_item_config("Left")
                    .selected(self.show_left)
                    .build()
                    .then(|| self.show_left = !self.show_left);
                ui.menu_item_config("ImGui Demo")
                    .selected(self.show_demo)
                    .build()
                    .then(|| self.show_demo = !self.show_demo);
            }
        }
    }
}