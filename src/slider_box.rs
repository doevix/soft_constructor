pub struct SliderBox;

use eframe::egui::{ Align2, Color32, FontFamily, FontId, Pos2, Rect, Stroke, Theme, Ui };
use eframe::epaint::{ CornerRadiusF32 };
use crate::physics::World;

impl SliderBox {
    pub fn init() -> Self {
        Self {

        }
    }

    pub fn draw(&self, ui: &mut Ui, area: Rect, world: World) {
        let vals = [
            (world.gravity, 4.0, " g"),
            (world.friction, 1.0, " f"),
            (world.springyness, 0.5, " k")];
        let divisions = vals.len();
        let div_width = area.width() / divisions as f32;

        let themed_stroke_color_val = if ui.theme() == Theme::Light { 0 } else { 128 };
        let themed_bg_color_val = if ui.theme() == Theme::Light { 255 } else { 16 };
        let themed_slider_color_val = if ui.theme() == Theme::Light { 180 } else { 64 };
        let stroke_color = Color32::from_gray(themed_stroke_color_val);
        let bg_color = Color32::from_gray(themed_bg_color_val);
        let slider_color = Color32::from_gray(themed_slider_color_val);

        ui.painter().rect_filled(area, CornerRadiusF32::ZERO, bg_color);

        let painter = ui.painter_at(area);
        let stroke = Stroke::new(1.0, stroke_color);
        for idx in 0..divisions {
            let val = vals[idx];
            let val_ratio = val.0 / val.1;

            let lower = Pos2::new(area.min.x + div_width * (idx + 1) as f32, area.max.y);
            let upper = Pos2::new(area.min.x + div_width * idx as f32, area.max.y - (area.size().y - div_width) * val_ratio as f32);
            let slider_rect = Rect::from_two_pos(upper, lower);

            let text_pos_x = 0.5 * div_width * (2.0 * (idx as f32) + 1.0);
            let text_pos_y = 0.5 * div_width + area.min.y;
            let text_pos = Pos2::new(text_pos_x, text_pos_y);

            painter.text(text_pos, Align2::LEFT_CENTER, String::from(val.2), FontId::new(18.0, FontFamily::default()), stroke_color);
            painter.rect_filled(slider_rect, CornerRadiusF32::ZERO, slider_color);
        }

        for div in 1..divisions {
            let div_x = area.min.x + div_width * div as f32;
            let p1 = Pos2::new(div_x, area.min.y);
            let p2 = Pos2::new(div_x, area.max.y);
            painter.line_segment([p1, p2], stroke);
        }

        painter.line_segment([
            area.min,
            Pos2::new(area.max.x, area.min.y)
        ], Stroke::new(2.0, stroke_color));
        painter.line_segment([
            Pos2::new(area.min.x, area.min.y + div_width),
            Pos2::new(area.max.x, area.min.y + div_width)
        ], stroke);
    }

    pub fn interact(&self, ui: &mut Ui, area: Rect, world: &mut World) {
        let vals = [
            (world.gravity, 4.0),
            (world.friction, 1.0),
            (world.springyness, 0.5),
        ];
        let response = ui.response();
        if response.dragged() {

        }
    }
}
