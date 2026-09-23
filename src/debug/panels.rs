use egui::{Align2, Color32, FontId, RichText, Sense, Shape, Stroke, Ui, pos2, vec2};
use glam::Vec3;

use crate::camera::CameraExposureSettings;
use crate::graphics::lighting::DirectionalLight;
use crate::graphics::pass::exposure::ExposureSettings;
use crate::graphics::{GpuVoxelTraceSettings, VOXEL_MIP_LEVEL_COUNT};

use super::{
    DebugInput,
    data::{DebugSnapshot, ProfileStage, VoxelDebugMode},
    history::ProfilerHistory,
};

const GRUVBOX_BG: Color32 = Color32::from_rgb(40, 40, 40);
const GRUVBOX_BG1: Color32 = Color32::from_rgb(60, 56, 54);
const GRUVBOX_BG2: Color32 = Color32::from_rgb(80, 73, 69);
const GRUVBOX_BG3: Color32 = Color32::from_rgb(102, 92, 84);
const GRUVBOX_FG: Color32 = Color32::from_rgb(235, 219, 178);
const GRUVBOX_GRAY: Color32 = Color32::from_rgb(146, 131, 116);
const GRUVBOX_RED: Color32 = Color32::from_rgb(251, 73, 52);
const GRUVBOX_GREEN: Color32 = Color32::from_rgb(184, 187, 38);
const GRUVBOX_YELLOW: Color32 = Color32::from_rgb(250, 189, 47);
const GRUVBOX_BLUE: Color32 = Color32::from_rgb(131, 165, 152);
const GRUVBOX_PURPLE: Color32 = Color32::from_rgb(211, 134, 155);

pub fn show(
    ui: &mut Ui,
    visible: bool,
    snapshot: &DebugSnapshot,
    input: &mut DebugInput<'_>,
    profiler: &ProfilerHistory,
    profiler_visible: &mut bool,
) {
    if visible {
        egui::Window::new("Debug")
            .default_width(340.0)
            .show(ui.ctx(), |ui| {
                ui.heading("Renderer");
                ui.label(format!("FPS: {:.1}", profiler.fps_average()));
                ui.label(format!(
                    "Frame time: {:.2} ms",
                    snapshot.delta_time * 1000.0
                ));
                ui.label(format!("Meshes: {}", snapshot.mesh_count));
                ui.label(format!("Draw calls: {}", snapshot.draw_count));
                if ui.button("Open profiler").clicked() {
                    *profiler_visible = true;
                }
                ui.separator();
                ui.heading("Camera");
                ui.label(format!(
                    "Position: ({:.2}, {:.2}, {:.2})",
                    snapshot.camera_position.x,
                    snapshot.camera_position.y,
                    snapshot.camera_position.z,
                ));
                show_camera_exposure(ui, input.camera_exposure);

                ui.separator();
                show_directional_light(ui, input.light);

                ui.separator();
                show_exposure(
                    ui,
                    input.exposure,
                    input.auto_exposure_enabled,
                    input.manual_exposure,
                );

                ui.separator();
                show_voxel_debug(
                    ui,
                    input.voxel_debug_enabled,
                    input.occupied_voxel_debug_enabled,
                    input.voxel_debug_mode,
                    input.mip_debug_enabled,
                    input.mip_level,
                    input.voxel_trace_settings,
                );
            });
    }

    if *profiler_visible {
        egui::Window::new("Profiler")
            .open(profiler_visible)
            .default_size([920.0, 560.0])
            .min_size([760.0, 380.0])
            .vscroll(true)
            .show(ui.ctx(), |ui| {
                ui.style_mut().visuals = gruvbox_visuals();
                show_profiler(ui, profiler);
            });
}

fn gruvbox_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(GRUVBOX_FG);
    visuals.weak_text_color = Some(GRUVBOX_GRAY);
    visuals.hyperlink_color = GRUVBOX_BLUE;
    visuals.faint_bg_color = GRUVBOX_BG1;
    visuals.extreme_bg_color = GRUVBOX_BG;
    visuals.code_bg_color = GRUVBOX_BG1;
    visuals.warn_fg_color = GRUVBOX_YELLOW;
    visuals.error_fg_color = GRUVBOX_RED;
    visuals.window_fill = GRUVBOX_BG;
    visuals.panel_fill = GRUVBOX_BG;
    visuals.window_stroke = Stroke::new(1.0, GRUVBOX_BG3);
    visuals.selection.bg_fill = GRUVBOX_BG3;
    visuals.selection.stroke = Stroke::new(1.0, GRUVBOX_FG);

    visuals.widgets.noninteractive.bg_fill = GRUVBOX_BG;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, GRUVBOX_FG);
    visuals.widgets.inactive.bg_fill = GRUVBOX_BG1;
    visuals.widgets.inactive.weak_bg_fill = GRUVBOX_BG1;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, GRUVBOX_BG3);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, GRUVBOX_FG);
    visuals.widgets.hovered.bg_fill = GRUVBOX_BG2;
    visuals.widgets.hovered.weak_bg_fill = GRUVBOX_BG2;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, GRUVBOX_YELLOW);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, GRUVBOX_FG);
    visuals.widgets.active.bg_fill = GRUVBOX_BG3;
    visuals.widgets.active.weak_bg_fill = GRUVBOX_BG3;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, GRUVBOX_YELLOW);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, GRUVBOX_FG);
    visuals
}
}

fn show_profiler(ui: &mut Ui, profiler: &ProfilerHistory) {
    ui.horizontal_wrapped(|ui| {
        let latest = profiler.latest();
        let fps = profiler.fps_average();
        let frame_ms = latest.map_or(0.0, |sample| sample.frame_time_ms);
        let cpu_ms = latest.map_or(0.0, |sample| sample.cpu_ms.iter().sum());
        let gpu_ms = latest.map_or(0.0, |sample| sample.gpu_ms.iter().sum());

        metric_card(
            ui,
            "AVG FPS",
            &format!("{fps:.1}"),
            GRUVBOX_GREEN,
        );
        metric_card(
            ui,
            "FRAME",
            &format!("{frame_ms:.2} ms"),
            frame_color(frame_ms),
        );
        metric_card(
            ui,
            "CPU WORK",
            &format!("{cpu_ms:.2} ms"),
            GRUVBOX_BLUE,
        );
        if profiler.gpu_supported() {
            metric_card(
                ui,
                "GPU",
                &format!("{gpu_ms:.2} ms"),
                GRUVBOX_PURPLE,
            );
        } else {
            metric_card(ui, "GPU", "Unavailable", GRUVBOX_GRAY);
        }
    });

    ui.add_space(8.0);
    ui.heading("Frame history");
    ui.label("Rolling 5 seconds · target 16.67 ms at 60 FPS");
    draw_frame_graph(ui, profiler);
    ui.horizontal(|ui| {
        graph_legend(ui, GRUVBOX_GREEN, "Frame interval");
        if profiler.gpu_supported() {
            graph_legend(ui, GRUVBOX_PURPLE, "GPU pass total");
        }
    });

    ui.add_space(8.0);
    ui.heading("Frame breakdown");
    ui.label("CPU time measures work on the main thread. GPU results arrive asynchronously.");
    if !profiler.gpu_supported() {
        ui.weak("GPU timestamps are unavailable on this graphics adapter.");
    }
    egui::Grid::new("profiler_breakdown")
        .num_columns(7)
        .striped(true)
        .spacing([12.0, 5.0])
        .show(ui, |ui| {
            ui.strong("Stage");
            ui.strong("CPU now");
            ui.strong("CPU avg 5s");
            ui.strong("CPU peak 5s");
            ui.strong("GPU now");
            ui.strong("GPU avg 5s");
            ui.strong("GPU peak 5s");
            ui.end_row();

            for stage in ProfileStage::ALL {
                let cpu = profiler
                    .latest()
                    .map_or(0.0, |sample| sample.cpu_ms[stage as usize]);
                let (cpu_avg, cpu_peak) = profiler.cpu_stats(stage);
                ui.label(stage.name());
                ui.label(format!("{cpu:.3} ms"));
                ui.label(format!("{cpu_avg:.3} ms"));
                ui.label(format!("{cpu_peak:.3} ms"));
                if profiler.gpu_supported() && stage as usize >= ProfileStage::RENDER_START as usize
                {
                    let gpu = profiler
                        .latest()
                        .map_or(0.0, |sample| sample.gpu_ms[stage as usize]);
                    let (gpu_avg, gpu_peak) = profiler.gpu_stats(stage);
                    ui.label(format!("{gpu:.3} ms"));
                    ui.label(format!("{gpu_avg:.3} ms"));
                    ui.label(format!("{gpu_peak:.3} ms"));
                } else {
                    ui.weak("—");
                    ui.weak("—");
                    ui.weak("—");
                }
                ui.end_row();
            }
        });
}

fn metric_card(ui: &mut Ui, title: &str, value: &str, color: Color32) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.set_width(130.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(title).small().color(GRUVBOX_GRAY));
                ui.label(RichText::new(value).strong().color(color));
            });
        });
}

fn draw_frame_graph(ui: &mut Ui, profiler: &ProfilerHistory) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 170.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);

    let graph = rect.shrink2(vec2(34.0, 12.0));
    let max_ms = profiler
        .samples()
        .map(|sample| sample.frame_time_ms.max(sample.gpu_ms.iter().sum()))
        .fold(33.34_f32, f32::max)
        .min(100.0);

    for target_ms in [16.67, 33.33] {
        let y = graph.bottom() - graph.height() * (target_ms / max_ms);
        painter.line_segment(
            [pos2(graph.left(), y), pos2(graph.right(), y)],
            Stroke::new(1.0, GRUVBOX_BG3),
        );
        painter.text(
            pos2(rect.left() + 3.0, y),
            Align2::LEFT_CENTER,
            format!("{target_ms:.0}"),
            FontId::proportional(10.0),
            GRUVBOX_GRAY,
        );
    }

    let samples: Vec<_> = profiler.samples().collect();
    if samples.len() < 2 {
        painter.text(
            graph.center(),
            Align2::CENTER_CENTER,
            "Collecting frame samples…",
            FontId::proportional(13.0),
            GRUVBOX_GRAY,
        );
        return;
    }

    let first_time = samples.first().unwrap().time;
    let last_time = samples.last().unwrap().time.max(first_time + f64::EPSILON);
    let points: Vec<_> = samples
        .iter()
        .map(|sample| {
            let x = graph.left()
                + graph.width() * ((sample.time - first_time) / (last_time - first_time)) as f32;
            let y =
                graph.bottom() - graph.height() * (sample.frame_time_ms / max_ms).clamp(0.0, 1.0);
            pos2(x, y)
        })
        .collect();
    painter.add(Shape::line(
        points,
        Stroke::new(1.8, GRUVBOX_GREEN),
    ));
    if profiler.gpu_supported() {
        let gpu_points: Vec<_> = samples
            .iter()
            .map(|sample| {
                let x = graph.left()
                    + graph.width()
                        * ((sample.time - first_time) / (last_time - first_time)) as f32;
                let gpu_ms: f32 = sample.gpu_ms.iter().sum();
                let y = graph.bottom() - graph.height() * (gpu_ms / max_ms).clamp(0.0, 1.0);
                pos2(x, y)
            })
            .collect();
        painter.add(Shape::line(
            gpu_points,
            Stroke::new(1.8, GRUVBOX_PURPLE),
        ));
    }
    painter.text(
        pos2(graph.left(), rect.bottom() - 2.0),
        Align2::LEFT_BOTTOM,
        "0 s",
        FontId::proportional(10.0),
        GRUVBOX_GRAY,
    );
    painter.text(
        pos2(graph.right(), rect.bottom() - 2.0),
        Align2::RIGHT_BOTTOM,
        "5 s",
        FontId::proportional(10.0),
        GRUVBOX_GRAY,
    );
}

fn graph_legend(ui: &mut Ui, color: Color32, label: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, color);
        ui.small(label);
    });
}

fn frame_color(milliseconds: f32) -> Color32 {
    if milliseconds <= 16.67 {
        GRUVBOX_GREEN
    } else if milliseconds <= 33.33 {
        GRUVBOX_YELLOW
    } else {
        GRUVBOX_RED
    }
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

fn show_exposure(
    ui: &mut Ui,
    exposure: &mut ExposureSettings,
    enabled: &mut bool,
    manual_exposure: &mut f32,
) {
    ui.collapsing("Auto exposure", |ui| {
        ui.checkbox(enabled, "Enable auto exposure");

        if *enabled {
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
                egui::Slider::new(&mut exposure.adaptation_speed, 0.1..=10.0)
                    .text("Adaptation speed"),
            );

            if ui.button("Reset auto exposure").clicked() {
                *exposure = ExposureSettings::default();
            }
        } else {
            ui.add(
                egui::Slider::new(manual_exposure, -12.0..=12.0)
                    .text("Manual exposure")
                    .suffix(" stops"),
            );

            if ui.button("Reset manual exposure").clicked() {
                *manual_exposure = 0.0;
            }
        }
    });
}

fn show_voxel_debug(
    ui: &mut Ui,
    brick_edges_enabled: &mut bool,
    occupied_voxels_enabled: &mut bool,
    voxel_debug_mode: &mut VoxelDebugMode,
    mipmap_enabled: &mut bool,
    mipmap_level: &mut u32,
    trace_settings: &mut GpuVoxelTraceSettings,
) {
    ui.collapsing("Voxel debug", |ui| {
        ui.checkbox(brick_edges_enabled, "Show brick edges");
        ui.checkbox(occupied_voxels_enabled, "Show occupied voxels");

        if *occupied_voxels_enabled {
            ui.indent("voxel-mode", |ui| {
                ui.radio_value(voxel_debug_mode, VoxelDebugMode::WhiteEdges, "White edges");
                ui.radio_value(
                    voxel_debug_mode,
                    VoxelDebugMode::AlbedoFaces,
                    "Solid faces with albedo",
                );
                ui.radio_value(
                    voxel_debug_mode,
                    VoxelDebugMode::NormalFaces,
                    "Solid faces with normals",
                );
                ui.radio_value(
                    voxel_debug_mode,
                    VoxelDebugMode::RadianceFaces,
                    "Solid faces with radiance",
                );
            });
        }

        ui.separator();
        ui.checkbox(mipmap_enabled, "Show voxel mipmaps");

        if *mipmap_enabled {
            ui.add(
                egui::Slider::new(mipmap_level, 0..=(VOXEL_MIP_LEVEL_COUNT as u32 - 1))
                    .text("Mipmap level")
                    .integer(),
            );
            ui.small("0-2: brick, 3-4: cluster, 5-7: global");
        }

        ui.separator();
        ui.collapsing("Cone tracing", |ui| {
            ui.add(
                egui::Slider::new(&mut trace_settings.cone_count, 3..=6)
                    .text("Cones")
                    .integer(),
            );
            ui.add(
                egui::Slider::new(&mut trace_settings.max_distance, 4.0..=100.0)
                    .text("Max distance"),
            );

            let mut cone_angle_degrees = trace_settings.cone_angle.to_degrees();
            if ui
                .add(
                    egui::Slider::new(&mut cone_angle_degrees, 10.0..=60.0)
                        .text("Cone angle")
                        .suffix("°"),
                )
                .changed()
            {
                trace_settings.cone_angle = cone_angle_degrees.to_radians();
            }

            ui.add(egui::Slider::new(&mut trace_settings.intensity, 0.0..=2.0).text("Intensity"));
        });
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
