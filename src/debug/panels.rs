use egui::Ui;

use super::data::DebugSnapshot;

pub fn show(ui: &mut Ui, visible: bool, snapshot: &DebugSnapshot) {
    if !visible {
        return;
    }

    egui::Window::new("Debug")
        .default_width(260.0)
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
        });
}

fn fps(delta_time: f32) -> f32 {
    if delta_time > 0.0 {
        1.0 / delta_time
    } else {
        0.0
    }
}
