use std::f32::consts::TAU;

use eframe::{egui::{ Color32, Pos2, Rect, Stroke, Theme, Ui }, epaint::CornerRadiusF32};
use crate::physics::{Muscle, Wave};

pub struct WaveBox {

}

impl WaveBox {
    pub fn init() -> Self {
        Self {

        }
    }

    pub fn draw(&self, ui: &mut Ui, area_rect: Rect, wave: Wave, muscles: &Vec<Muscle>) {
        let themed_bg_color_val = if ui.theme() == Theme::Light { 255 } else { 16 };
        let themed_stroke_color_val = if ui.theme() == Theme::Light { 0 } else { 128 };
        let themed_stroke_dim_color_val = if ui.theme() == Theme::Light { 128 } else { 64 };

        let wave_bg_color = Color32::from_gray(themed_bg_color_val);
        let wave_stroke_color = Color32::from_gray(themed_stroke_color_val);
        let dimmed_stroke_color = Color32::from_gray(themed_stroke_dim_color_val);
        let stroke = Stroke::new(1.0, wave_stroke_color);
        let dimmed_stroke = Stroke::new(1.0, dimmed_stroke_color);
        ui.painter().rect_filled(area_rect, CornerRadiusF32::same(0.0), wave_bg_color);


        let painter = ui.painter_at(area_rect);

        let wave_limit_division = area_rect.size().x / 8.0;
        let wave_left_limit = area_rect.min.x + wave_limit_division;
        let wave_right_limit = area_rect.max.x - wave_limit_division;
        let wave_bottom_box_limit = area_rect.size().y / 5.0;


        let wave_rect = Rect::from_min_max(
            Pos2::new(wave_left_limit, area_rect.min.y),
            Pos2::new(wave_right_limit, area_rect.max.y - wave_bottom_box_limit),
        );

        let n_points = 60;
        let mut point_coords: Vec<Pos2> = Vec::new();
        let y_div_len = wave_rect.size().y / n_points as f32;

        // Draw the muscles
        for msl in muscles {
            let y_factor = msl.phase as f32 / TAU;
            let y_coord = y_factor * wave_rect.size().y + wave_rect.min.y;
            let p1 = Pos2::new(wave_rect.min.x, y_coord);
            let p2 = Pos2::new(wave_rect.max.x, y_coord);
            painter.line_segment([p1, p2], dimmed_stroke);

            let sense_coord = Pos2::new(
                wave_rect.min.x + msl.sense as f32 * wave_rect.size().x,
                y_coord);
            painter.circle_filled(sense_coord, 2.0, dimmed_stroke_color);
        }

        // Get the coordinates of each point for the wave.
        for point in 0..n_points + 1 {
            let y_coord = point as f32 * y_div_len;
            let phase = TAU * y_coord / wave_rect.size().y;
            let x_coord = wave.output(0.5 * wave_rect.size().x as f64, phase as f64) as f32;
            let x_offset = wave_rect.min.x + wave_rect.size().x * 0.5;

            point_coords.push(Pos2::new(x_coord + x_offset, y_coord + wave_rect.min.y));
        }

        // Connect the points with segments and draw the wave.
        for (idx, point) in point_coords.iter().enumerate() {
            if idx + 1 < point_coords.len() {
                let next_point = point_coords.as_slice()[idx + 1];
                painter.line_segment([*point, next_point], stroke);
            }
        }

        // Draw the wave speed indicator.
        let speed_height = wave_rect.max.y * wave.speed as f32 / 0.1;
        let speed_indicator_rect = Rect::from_min_max(
            area_rect.min,
            Pos2::new(wave_rect.min.x, speed_height)
        );
        painter.rect_filled(speed_indicator_rect, CornerRadiusF32::same(0.0), dimmed_stroke_color);

        // Draw a muscle position guide.
        let divs = 4;
        let seg_len: f32 = wave_rect.size().y / divs as f32;
        for division in 1..divs {
            let y_coord = wave_rect.min.y + division as f32 * seg_len;
            let p1 = Pos2::new(wave_rect.max.x, y_coord);
            let p2 = Pos2::new(area_rect.max.x, y_coord);
            painter.line_segment([p1, p2], dimmed_stroke);
        }

        // Draw the wavebox's borders.
        painter.line_segment([wave_rect.min, Pos2::new(wave_rect.min.x, wave_rect.max.y)], dimmed_stroke);
        painter.line_segment([Pos2::new(wave_rect.max.x, wave_rect.min.y), wave_rect.max], dimmed_stroke);
        painter.line_segment(
            [Pos2::new(area_rect.min.x, wave_rect.max.y), Pos2::new(area_rect.max.x, wave_rect.max.y)],
            dimmed_stroke
        );
    }
}
