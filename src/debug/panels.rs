use egui::Ui;
use glam::Vec3;

use crate::graphics::lighting::DirectionalLight;

use super::data::DebugSnapshot;

pub fn show(ui: &mut Ui, visible: bool, snapshot: &DebugSnapshot, light: &mut DirectionalLight) {
    if !visible {
        return;
    }

    egui::Window::new("Debug")
        .default_width(320.0)
        .show(ui.ctx(), |ui| {
            ui.heading("Renderer");
            ui.label(format!("FPS: {:.1}", fps(snapshot.delta_time)));
            ui.label(format!(
                "Frame time: {:.2} ms",
                snapshot.delta_time * 1000.0
            ));
            ui.label(format!("Meshes: {}", snapshot.mesh_count));
            ui.label(format!("Draw calls: {}", snapshot.draw_count));

            ui.separator();
            ui.heading("Camera");
            ui.label(format!(
                "Position: ({:.2}, {:.2}, {:.2})",
                snapshot.camera_position.x, snapshot.camera_position.y, snapshot.camera_position.z,
            ));

            ui.separator();
            show_directional_light(ui, light);
        });
}

fn show_directional_light(ui: &mut Ui, light: &mut DirectionalLight) {
    ui.collapsing("Directional light", |ui| {
        ui.label("Direction");

        let mut direction = light.direction.to_array();
        let direction_changed = ui
            .add(egui::Slider::new(&mut direction[0], -1.0..=1.0).text("X"))
            .changed()
            | ui.add(egui::Slider::new(&mut direction[1], -1.0..=1.0).text("Y"))
                .changed()
            | ui.add(egui::Slider::new(&mut direction[2], -1.0..=1.0).text("Z"))
                .changed();

        if direction_changed {
            light.direction = Vec3::from_array(direction);
        }

        ui.label(format!(
            "Vector: ({:.2}, {:.2}, {:.2})",
            light.direction.x, light.direction.y, light.direction.z,
        ));
        ui.small("Direction is normalized in the shader.");

        ui.add(egui::Slider::new(&mut light.intensity, 0.0..=5.0).text("Intensity"));
        ui.add(egui::Slider::new(&mut light.ambient, 0.0..=1.0).text("Ambient"));

        let mut color = light.color.to_array();
        ui.horizontal(|ui| {
            ui.label("Color");
            if egui::color_picker::color_edit_button_rgb(ui, &mut color).changed() {
                light.color = Vec3::from_array(color);
            }
        });

        if ui.button("Reset").clicked() {
            *light = DirectionalLight::default();
        }
    });
}

fn fps(delta_time: f32) -> f32 {
    if delta_time > 0.0 {
        1.0 / delta_time
    } else {
        0.0
    }
}
