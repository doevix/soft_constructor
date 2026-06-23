use std::f32::consts::TAU;

use eframe::{egui::{ Color32, Pos2, Rect, Stroke, Theme, Ui }, epaint::CornerRadiusF32};
use crate::physics::Wave;

pub struct WaveBox {

}

impl WaveBox {
    pub fn init() -> Self {
        Self {

        }
    }

    pub fn draw(&self, ui: &mut Ui, area_rect: Rect, wave: Wave) {
        let themed_bg_color_val = if ui.theme() == Theme::Light { 255 } else { 16 };
        let themed_stroke_color_val = if ui.theme() == Theme::Light { 0 } else { 128 };

        let wave_bg_color = Color32::from_gray(themed_bg_color_val);
        let wave_stroke_color = Color32::from_gray(themed_stroke_color_val);
        let stroke = Stroke::new(1.0, wave_stroke_color);
        ui.painter().rect_filled(area_rect, CornerRadiusF32::same(0.0), wave_bg_color);

        let n_points = 60;
        let mut point_coords: Vec<Pos2> = Vec::new();
        let y_div_len = area_rect.size().y / n_points as f32;
        for point in 0..n_points + 1 {
            let y_coord = point as f32 * y_div_len;
            let phase = TAU * y_coord / area_rect.size().y;
            let x_coord = wave.output(0.5 * area_rect.size().x as f64, phase as f64) as f32;
            let x_offset = area_rect.min.x + area_rect.size().x * 0.5;

            point_coords.push(Pos2::new(x_coord + x_offset, y_coord + area_rect.min.y));
        }

        let painter = ui.painter_at(area_rect);
        for (idx, point) in point_coords.iter().enumerate() {
            if idx < point_coords.len() - 1 {
                let next_point = point_coords.as_slice()[idx + 1];
                painter.line_segment([*point, next_point], stroke);
            }
        }
    }
}
