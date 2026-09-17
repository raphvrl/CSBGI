use egui::Ui;
use glam::Vec3;

use crate::camera::CameraExposureSettings;
use crate::graphics::lighting::DirectionalLight;
use crate::graphics::pass::exposure::ExposureSettings;

use super::data::DebugSnapshot;

pub fn show(
    ui: &mut Ui,
    visible: bool,
    snapshot: &DebugSnapshot,
    light: &mut DirectionalLight,
    exposure: &mut ExposureSettings,
    camera_exposure: &mut CameraExposureSettings,
) {
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
            show_camera_exposure(ui, camera_exposure);

            ui.separator();
            show_directional_light(ui, light);

            ui.separator();
            show_exposure(ui, exposure);
        });
}

fn show_camera_exposure(ui: &mut Ui, camera: &mut CameraExposureSettings) {
    ui.collapsing("Physical camera", |ui| {
        ui.add(egui::Slider::new(&mut camera.aperture, 1.0..=32.0).text("Aperture f/"));

        let mut shutter_denominator = (1.0 / camera.shutter_speed_seconds).round();
        if ui
            .add(
                egui::DragValue::new(&mut shutter_denominator)
                    .speed(1.0)
                    .range(1.0..=2000.0)
                    .prefix("Shutter: 1/")
                    .suffix(" s"),
            )
            .changed()
        {
            camera.shutter_speed_seconds = 1.0 / shutter_denominator.max(1.0);
        }

        ui.add(
            egui::Slider::new(&mut camera.iso, 50.0..=3200.0)
                .logarithmic(true)
                .text("ISO"),
        );
        ui.add(
            egui::Slider::new(&mut camera.exposure_compensation, -4.0..=4.0)
                .text("Compensation")
                .suffix(" EV"),
        );
        ui.label(format!("EV100: {:.2}", camera.ev100()));

        if ui.button("Reset camera").clicked() {
            *camera = CameraExposureSettings::default();
        }
    });
}

fn show_exposure(ui: &mut Ui, exposure: &mut ExposureSettings) {
    ui.collapsing("Auto exposure", |ui| {
        ui.add(
            egui::Slider::new(&mut exposure.target_luminance, 0.01..=1.0)
                .logarithmic(true)
                .text("Target luminance"),
        );
        ui.add(
            egui::Slider::new(&mut exposure.min_exposure, -12.0..=0.0)
                .text("Minimum exposure")
                .suffix(" stops"),
        );
        ui.add(
            egui::Slider::new(&mut exposure.max_exposure, 0.0..=12.0)
                .text("Maximum exposure")
                .suffix(" stops"),
        );
        ui.add(
            egui::Slider::new(&mut exposure.adaptation_speed, 0.1..=10.0).text("Adaptation speed"),
        );

        if ui.button("Reset").clicked() {
            *exposure = ExposureSettings::default();
        }
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
                    .suffix("°")
                    .integer(),
            )
            .changed()
            | ui.add(
                egui::Slider::new(&mut yaw_degrees, -180.0..=180.0)
                    .text("Yaw")
                    .suffix("°")
                    .integer(),
            )
            .changed();

        if angles_changed {
            light.set_sun_angles(rise_degrees.to_radians(), yaw_degrees.to_radians());
        }

        ui.small("-90° = sunrise, 0° = noon, 90° = sunset");

        ui.add(
            egui::DragValue::new(&mut light.illuminance_lux)
                .speed(1_000.0)
                .range(0.0..=200_000.0)
                .prefix("Illuminance: ")
                .suffix(" lux"),
        );
        ui.add(
            egui::DragValue::new(&mut light.ambient_illuminance_lux)
                .speed(100.0)
                .range(0.0..=100_000.0)
                .prefix("Ambient: ")
                .suffix(" lux"),
        );

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
