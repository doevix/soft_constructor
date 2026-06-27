pub mod v2d;
pub mod physics;
pub mod model_io;
pub mod wavebox;
pub mod slider_box;

use std::f64::consts::TAU;
use std::{time::Instant};

use v2d::V2D;
use eframe::egui::{self, Color32, ComboBox, MenuBar, Pos2, Rect, UiBuilder, Vec2 };
use eframe::epaint::{ CornerRadiusF32, Stroke };
use rfd::FileDialog;

use crate::physics::{ GravityDirection, Mass, Model, Spring, SurfaceSticky, Wave, WaveDirection, World };
use crate::slider_box::SliderBox;
use crate::wavebox::WaveBox;


const TICKS_PER_SEC: f64 = 300.0;

/*
 * Notes on sodaplay values:
 * max gravity: 4.0
 * max friction: 1.0
 * max springyness: 0.5
 * default surface friction: 0.1
 * default surface reflection: 0.75
 * default amplitude 0.5
 * max wave speed = 0.1 which is weird.
 */

#[derive(PartialEq, Eq)]
enum SimulationState {
    Simulate,
    Construct,
    Delete,
    ClearAll,
}

#[derive(PartialEq, Eq)]
enum WaveUserState {
    AutoReverse,
    Forward,
    Reverse,
    Manual,
}

fn main() {
    let loaded = load_model("test_models/daintywalker.xml");

    let (model, world, wave) = match loaded {
        Some((l_model, l_world, l_wave)) => { (l_model, l_world, l_wave) },
        None => {
            let empty_model = Model::new();
            let empty_world = World::new(830.0, 542.0, 0.2, 0.2, 0.5, 0.75, 0.1, GravityDirection::Down);
            let empty_wave = Wave::new(0.5, 0.1, 0.0, true, WaveDirection::Forward);

            (empty_model, empty_world, empty_wave)
        }
    };

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("Soft Constructor", native_options, Box::new(|cc| Ok(Box::new(ConstructorApp::new(cc, model, world, wave)))));
}

fn load_model(filename: &str) -> Option<(Model, World, Wave)> {

    let model_data_res = model_io::Loader::load(filename);

    let loaded = if let Ok(model_data) = model_data_res {
        let mut model = Model::new();

        let mut indexed_masses: Vec<(usize, Mass)> = Vec::new();

        // Load fixed masses.
        for node_dat in model_data.nodes.fixed_nodes {
            let pos = V2D::new(node_dat.x, node_dat.y);
            let vel = V2D::null();
            let m = Mass::new(pos, vel, node_dat.id.fixed);
            indexed_masses.push((node_dat.id.idx, m));
        }

        // Load free masses.
        for mass_dat in model_data.nodes.masses {
            let pos = V2D::new(mass_dat.x, mass_dat.y);
            let vel = V2D::new(mass_dat.vx, mass_dat.vy);
            let m = Mass::new(pos, vel, mass_dat.id.fixed);
            indexed_masses.push((mass_dat.id.idx, m));
        }

        indexed_masses.sort_by_key(|(idx, _)| *idx);

        for (_, mass) in indexed_masses {
            model.add_mass(mass);
        }

        for spring_dat in model_data.links.springs {
            let s = Spring::new(spring_dat.a, spring_dat.b, spring_dat.restlength);
            model.add_spring(s);
        }

        for muscle_dat in model_data.links.muscles {
            let msl = Spring::new(muscle_dat.a, muscle_dat.b, muscle_dat.restlength);

            let spring_count = model.add_spring(msl);
            model.attach_muscle(spring_count - 1, muscle_dat.amplitude, TAU * muscle_dat.phase);
        }

        let gravity_direction =
            if model_data.settings.gravitydirection == "up" { GravityDirection::Up }
            else if model_data.settings.gravitydirection == "down" { GravityDirection::Down }
            else { GravityDirection::Off };
        let world = World::new(
            model_data.container.width, model_data.container.height,
            model_data.environment.gravity, model_data.environment.friction, model_data.environment.springyness,
            model_data.collisions.surface_reflection.abs(), model_data.collisions.surface_friction, gravity_direction
        );
        let autoreverse = if model_data.settings.autoreverse == "on" { true } else { false };
        let wave_direction = if model_data.settings.wavedirection == "forward" { WaveDirection::Forward } else { WaveDirection::Reverse };
        let wave = Wave::new(model_data.wave.amplitude, model_data.wave.speed, model_data.wave.phase, autoreverse, wave_direction);

        Some((model, world, wave))
    } else { None };

    loaded
}

struct ConstructorApp {
    pub last_frame: f64,
    pub t_now: Instant,
    pub model: Model,
    pub world: World,
    pub wave: Wave,
    pub acc: f64,
    pub sim_state: SimulationState,
    pub wavebox: WaveBox,
    pub slider_box: SliderBox,
    pub rect_viewport: Rect,
}

impl ConstructorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, model: Model, world: World, wave: Wave) -> Self {
        Self {
            last_frame: 0.0,
            t_now: Instant::now(),
            model, world, wave,
            acc: 0.0,
            sim_state: SimulationState::Simulate,
            wavebox: WaveBox::init(),
            slider_box: SliderBox::init(),
            rect_viewport: Rect::ZERO,
        }
    }
    pub fn to_panel(&self, scale: f32, rect: Rect, v2_in: V2D) -> Pos2 {

        Pos2 {
            x: v2_in.x as f32 * scale + rect.min.x,
            y: (self.world.height - v2_in.y) as f32 * scale + rect.min.y,
        }
    }
}

impl eframe::App for ConstructorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("menu").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load").clicked() {
                        let files = FileDialog::new()
                        .add_filter("text", &["txt", "xml"])
                        .add_filter("xml", &["txt", "xml"])
                        .pick_file();

                        if let Some(file) = files {
                            let filename = file.to_str().unwrap_or("");
                            let loaded = load_model(filename);
                            if let Some(to_load) = loaded {
                                (self.model, self.world, self.wave) = to_load;
                                self.acc = 0.0;
                            }
                        }
                    }
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                egui::widgets::global_theme_preference_buttons(ui);

                let sim_selects = [
                    (SimulationState::Simulate, "simulate"),
                    (SimulationState::Construct, "construct"),
                    (SimulationState::Delete, "delete"),
                    (SimulationState::ClearAll, "clear all"),
                ];

                let wave_selects = [
                    (WaveUserState::AutoReverse, "auto reverse"),
                    (WaveUserState::Forward, "forward"),
                    (WaveUserState::Reverse, "reverse"),
                    (WaveUserState::Manual, "manual"),
                ];

                let mut current_wave_mode = if self.wave.autoreverse {
                    WaveUserState::AutoReverse
                } else {
                    match self.wave.direction {
                        WaveDirection::Forward => WaveUserState::Forward,
                        WaveDirection::Reverse=> WaveUserState::Reverse,
                        WaveDirection::Manual => WaveUserState::Manual,
                    }
                };

                let gravity_selects = [
                    (GravityDirection::Down, "gravity on"),
                    (GravityDirection::Off, "gravity off"),
                    (GravityDirection::Up, "gravity reverse"),
                ];

                let mut current_gravity_dir = self.world.gravity_direction;

                let surface_selects = [
                    (SurfaceSticky::Sticky, "sticky"),
                    (SurfaceSticky::Slippy, "slippery"),
                ];
                let mut current_surface = self.world.stickyness;

                ui.horizontal(|ui| {
                    ui.columns(4, |columns| {
                        // Simulation mode options.
                        ComboBox::new("Simulation", "")
                        .width(columns[0].available_width())
                        .selected_text(sim_selects.iter().find(|(sim, _)| self.sim_state == *sim ).unwrap().1)
                        .show_ui(&mut columns[0], |ui| {
                            ui.selectable_value(&mut self.sim_state, SimulationState::Simulate, "simulate");
                            ui.selectable_value(&mut self.sim_state, SimulationState::Construct, "construct");
                            ui.selectable_value(&mut self.sim_state, SimulationState::Delete, "delete");
                            ui.selectable_value(&mut self.sim_state, SimulationState::ClearAll, "clear all");
                        });

                        // Wave direction options.
                        ComboBox::new("Wave", "")
                        .width(columns[1].available_width())
                        .selected_text(wave_selects.iter().find(|(wave, _)| current_wave_mode == *wave).unwrap().1)
                        .show_ui(&mut columns[1], |ui| {
                            ui.selectable_value(&mut current_wave_mode, WaveUserState::AutoReverse, "auto reverse");
                            ui.selectable_value(&mut current_wave_mode, WaveUserState::Forward, "forward");
                            ui.selectable_value(&mut current_wave_mode, WaveUserState::Reverse, "reverse");
                            ui.selectable_value(&mut current_wave_mode, WaveUserState::Manual, "manual");
                        });

                        match current_wave_mode {
                            WaveUserState::AutoReverse => {
                                self.wave.autoreverse = true;
                                self.model.reset_wall_hit();
                            },
                            WaveUserState::Forward => {
                                self.wave.autoreverse = false;
                                self.wave.direction = WaveDirection::Forward;
                            },
                            WaveUserState::Reverse => {
                                self.wave.autoreverse = false;
                                self.wave.direction = WaveDirection::Reverse;
                            },
                            WaveUserState::Manual => {
                                self.wave.autoreverse = false;
                                self.wave.direction = WaveDirection::Manual;
                            },
                        }

                        // Gravity direction options.
                        ComboBox::new("Gravity", "")
                        .width(columns[2].available_width())
                        .selected_text(gravity_selects.iter().find(|(grav, _)| current_gravity_dir == *grav).unwrap().1)
                        .show_ui(&mut columns[2], |ui| {
                            ui.selectable_value(&mut current_gravity_dir, GravityDirection::Down, "gravity on");
                            ui.selectable_value(&mut current_gravity_dir, GravityDirection::Off, "gravity off");
                            ui.selectable_value(&mut current_gravity_dir, GravityDirection::Up, "gravity reverse");
                        });
                        self.world.gravity_direction = current_gravity_dir;

                        // Surface friction options.
                        ComboBox::new("Surface", "")
                        .width(columns[3].available_width())
                        .selected_text(surface_selects.iter().find(|(surface, _)| current_surface == *surface).unwrap().1)
                        .show_ui(&mut columns[3], |ui| {
                            ui.selectable_value(&mut current_surface, SurfaceSticky::Sticky, "sticky");
                            ui.selectable_value(&mut current_surface, SurfaceSticky::Slippy, "slippery");
                        });
                        self.world.set_stickyness(current_surface);
                    });
                });
            });


        });
        egui::Panel::bottom("info").show_inside(ui, |ui| {
            let disp_frame_time = format!("{:.2?} Hz", self.last_frame.recip());
            ui.label(disp_frame_time);
        });
        egui::Panel::left("wave and settings")
        .exact_size(self.rect_viewport.width() * 0.12)
        .max_size(150.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.vertical(|ui| {
                let full_area = ui.max_rect();
                let total_height = full_area.height();

                let mut wave_area = full_area;
                wave_area.max.y = wave_area.min.y + (3.0 * total_height / 4.0);
                self.wavebox.draw(ui, wave_area, self.wave, self.model.get_muscles());

                let mut rect_area = full_area;
                rect_area.min.y = wave_area.max.y;
                let r_ui = UiBuilder::new().max_rect(rect_area);
                ui.scope_builder(r_ui, |ui| {
                    self.slider_box.draw(ui, rect_area, self.world);
                });
            });
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let t_elapsed = self.t_now.elapsed();
            self.last_frame = t_elapsed.as_secs_f64();
            self.t_now = Instant::now();

            self.acc += self.last_frame.min(0.25);
            let tick_interval = TICKS_PER_SEC.recip();
            while self.acc >= tick_interval {
                self.model.step(&mut self.wave, self.world);
                self.acc -= tick_interval;
            }
            let alpha = self.acc / tick_interval;

            ui.request_repaint();

            let t_value_elements = if ui.theme() == egui::Theme::Light { 0 } else { 128 };
            let t_value_bg = if ui.theme() == egui::Theme::Light { 255 } else { 16 };
            let t_value_empty = if ui.theme() == egui::Theme::Light { 128 } else { 0 };

            let bg_color = Color32::from_gray(t_value_bg);
            let mass_color = Color32::from_gray(t_value_elements);
            let spring_color = Color32::from_gray(t_value_elements);
            let empty_area_color = Color32::from_gray(t_value_empty);


            let panel_area = ui.max_rect();

            let rect_sz = panel_area.size();
            let (scaled_area, scale) = if self.world.width > self.world.height {
                let mut scale = rect_sz.x / (self.world.width as f32);
                let mut centered = Vec2::new(rect_sz.x, self.world.height as f32 * scale);
                if self.world.height as f32 * scale > rect_sz.y {
                    scale = rect_sz.y / (self.world.height as f32);
                    centered = Vec2::new(self.world.width as f32 * scale, rect_sz.y);
                }
                (centered, scale)
            } else {
                let mut scale = rect_sz.y / (self.world.height as f32);
                let mut centered = Vec2::new(self.world.width as f32 * scale, rect_sz.y);
                if self.world.width as f32 * scale > rect_sz.x {
                    scale = rect_sz.x / (self.world.width as f32);
                    centered = Vec2::new(rect_sz.x, (self.world.height as f32) * scale);
                }
                (centered, scale)
            };

            let center_offset = Vec2::new((panel_area.width() - scaled_area.x) / 2.0, (panel_area.height() - scaled_area.y) / 2.0);
            let centered_min = panel_area.min + center_offset;
            let centered_rect = Rect::from_min_size(centered_min, scaled_area);
            self.rect_viewport = centered_rect;

            ui.painter().rect_filled(panel_area, CornerRadiusF32::same(0.0), empty_area_color);
            ui.painter().rect_filled(centered_rect, CornerRadiusF32::same(0.0), bg_color);

            let painter = ui.painter_at(centered_rect);
            for spring in self.model.get_springs() {
            let spring_style = Stroke::new(1.0 * scale, spring_color);
                let m1 = self.model.get_mass(spring.a);
                let m2 = self.model.get_mass(spring.b);

                let p1 = self.to_panel(scale, centered_rect, m1.approx_pos(alpha));
                let p2 = self.to_panel(scale, centered_rect, m2.approx_pos(alpha));

                painter.line_segment([p1, p2], spring_style);
            }

            for muscle in self.model.get_muscles() {
                let spring = self.model.get_spring(muscle.spring_idx);
                let p1 = self.model.get_mass(spring.a).approx_pos(alpha);
                let p2 = self.model.get_mass(spring.b).approx_pos(alpha);
                let center = self.to_panel(scale, centered_rect, (p1 + p2) * 0.5);
                let rad = 1.0 * scale;

                painter.circle_filled(center, rad, mass_color);
            }

            for mass in self.model.get_masses() {
                let pos = self.to_panel(scale, centered_rect, mass.approx_pos(alpha));
                let rad: f32 = 2.5 * scale;
                if !mass.fixed {
                    painter.circle_filled(pos, rad, mass_color);
                } else {
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::new(5.0, 5.0) * scale), CornerRadiusF32::ZERO, mass_color);
                }
            }

        });
    }
}
