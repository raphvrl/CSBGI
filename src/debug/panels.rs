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
    ui.collapsing("Sun", |ui| {
        let (rise, yaw) = light.sun_angles();
        let mut rise_degrees = rise.to_degrees();
        let mut yaw_degrees = yaw.to_degrees();

        let angles_changed = ui
            .add(
                egui::Slider::new(&mut rise_degrees, -90.0..=90.0)
                    .text("Rise / set")
                    .suffix("°"),
            )
            .changed()
            | ui.add(
                egui::Slider::new(&mut yaw_degrees, -180.0..=180.0)
                    .text("Yaw")
                    .suffix("°"),
            )
            .changed();

        if angles_changed {
            light.set_sun_angles(rise_degrees.to_radians(), yaw_degrees.to_radians());
        }

        ui.small("-90° = sunrise, 0° = noon, 90° = sunset");

        ui.add(
            egui::DragValue::new(&mut light.intensity)
                .speed(0.1)
                .prefix("Intensity: "),
        );
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
