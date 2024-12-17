// Inspired by: https://github.com/EmbersArc/eplot/

use egui::{
   Ui, Color32, Align2, Vec2, vec2, Pos2, pos2, Shape, remap, Stroke, Resize, Rect, Sense, FontId, FontFamily, 
};


const RCENTER: Align2 = Align2::RIGHT_CENTER;

const COLORS: [Color32; 16] = [
                Color32::YELLOW,
                Color32::WHITE,
                Color32::LIGHT_YELLOW,
                Color32::LIGHT_BLUE,
                Color32::LIGHT_GREEN,
                Color32::LIGHT_RED,
                Color32::RED,
                Color32::DARK_RED,
                Color32::LIGHT_BLUE,
                Color32::BLUE,
                Color32::DARK_BLUE,
                Color32::BROWN,
                Color32::DARK_GREEN,
                Color32::GOLD,
                Color32::KHAKI,
                Color32::GREEN,
];



pub struct Memory {
   last_pos: Option<Pos2>,
   ys: f32,
   ye: f32,
}


impl Default for Memory {
   fn default() -> Self {
      Self {
         last_pos: None,
         ys:      -100.0,
         ye:       100.0,
      }
   }
}


pub struct Plot<'a> {
   mem:  &'a mut Memory,
   size: Vec2,
   show_yticks: bool,
   x0: f32,
   x1: f32,
}


impl<'a> Plot<'a> {
   pub fn new(mem: &'a mut Memory, size: Vec2, samples: usize, show_yticks: bool) -> Self {
      Self {
         mem,
         size,
         show_yticks,
         x0: 0.0,
         x1: samples as f32,
      }
   }

   pub fn set_y_range(&mut self, s: f32, e: f32) {
      self.mem.ys = s;
      self.mem.ye = e;
   }

   pub fn show(self, ui: &mut Ui, raw_data: &Vec<Pos2>, id: usize) {
      let id_string = format!("{}", id);
      Resize::default().id_salt(id_string).fixed_size(self.size).show(ui, |ui: &mut Ui| {
         let (response, mut painter) = ui.allocate_painter(ui.available_size(), Sense::drag());
         let canvas_rect = Rect::from_min_max(
            response.rect.min + vec2(if self.show_yticks { 40.0 } else { 0.0 }, 0.0),
            response.rect.max,
         );
         painter.rect(canvas_rect, 0.0, Color32::BLACK, Stroke::new(1.0, Color32::DARK_GRAY)); // plot the frame

         // Dragging
         if let Some(pos) = response.interact_pointer_pos() {
            let y_tf = remap(pos.y, canvas_rect.y_range(), self.mem.ye..=self.mem.ys);
            if let Some(p0) = self.mem.last_pos {
               let y0_tf = remap(p0.y, canvas_rect.y_range(), self.mem.ye..=self.mem.ys);
               let delta =  y0_tf - y_tf;
               self.mem.ys += delta;
               self.mem.ye += delta;
            }
            self.mem.last_pos = Some(pos);
         }

         // Zooming
         let scrolled = ui.input(|i| i.raw_scroll_delta.y.clamp(-10., 10.) );
         if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos().filter(|p| canvas_rect.contains(*p))) {
            if scrolled != 0. {
               let bottom_distance = (canvas_rect.bottom() - mouse_pos.y) / canvas_rect.height();
               let f = -0.01 * scrolled;
               let r = self.mem.ye - self.mem.ys;
               self.mem.ys -= f * bottom_distance * r;
               self.mem.ye += f * (1. - bottom_distance) * r;
            }
         }

         // Y-Axis ticks
         if self.show_yticks {
            let ticks = calc_tick4(self.mem.ys, self.mem.ye);
            for tick_y in ticks {
               let tk = Pos2 {
                  x: canvas_rect.min.x,
                  y: remap(tick_y, self.mem.ye..=self.mem.ys, canvas_rect.y_range()),
               };
               painter.line_segment([tk, tk + 5.0 * Vec2::X], Stroke::new(1.0, Color32::WHITE) );
               let font_id = FontId { size: 10.0, family:FontFamily::Proportional };
               painter.text(tk - 15.0 * Vec2::X, RCENTER, format!("{tick_y:.0}"), font_id, Color32::WHITE);
            }
         }

         painter.set_clip_rect(canvas_rect);
         let points: Vec<Pos2> = raw_data.iter().map(|p| {
            let x = remap(p.x, self.x0..=self.x1, canvas_rect.x_range());
            let y = remap(p.y, self.mem.ye..=self.mem.ys, canvas_rect.y_range());
            pos2(x, y)
         }).collect();
         let stroke = Stroke::new(1.0, COLORS[(id-1) % 16]);
         painter.add(Shape::line(points, stroke));
      })
   }
}


fn calc_tick4(start: f32, end: f32) -> Vec<f32> {
   let range = end - start;
   let step = range / 4.0;
   let mut tick_values: Vec<f32> = vec![];
   let mut i: f32 = start;
   loop {
      tick_values.push(i);
      i = i + step;
      if i > end { break;}
   }
   tick_values
}
