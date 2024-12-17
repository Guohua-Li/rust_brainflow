use std::sync::Arc;
use std::collections::HashSet;
use std::collections::HashMap;

use ndarray::Array2;

use egui::{
   CentralPanel, TopBottomPanel, Ui, Context, Response, Grid, Pos2, Vec2, vec2, Color32, Stroke, Resize, Rect,
   Button, FontId, RichText, FontFamily, FontData, FontDefinitions, Key, ViewportCommand, remap, Sense, Align2
};


use crate::eplot::{Memory, Plot};
use crate::chaninfo::ChanInfo;

use brainflow::{
   BoardIds,
   BrainFlowPresets,
   brainflow_input_params::BrainFlowInputParamsBuilder,
   board_shim::{ self, BoardShim, },
};

const BOARD:  BoardIds         = BoardIds::SyntheticBoard;
const PRESET: BrainFlowPresets = BrainFlowPresets::DefaultPreset;
const FS15:   f32  = 15.0;
const FS16:   f32  = 16.0;
const CCENTER: Align2 = Align2::CENTER_CENTER;
const COLS:   usize = 2;


pub struct MyApp {
   board:          BoardShim,
   is_streaming:   bool,
   samples:        usize,
   hash_map:       HashMap<String, Memory>,
   ui_size:        Vec2,
   plt_size:       Vec2,
   yaxis_on:       bool,
   show_chans:     Vec<Vec<usize>>,
   chan_ctrl:      ChanInfo,
   num_chans:      usize,
   adjust_y:       bool,
}


impl Default for MyApp {
   fn default() -> Self {
      Self {
         board: BoardShim::new(BOARD, BrainFlowInputParamsBuilder::default().build()).unwrap(),
         is_streaming:   false,
         samples:        1000,
         hash_map:       HashMap::new(),
         ui_size:        vec2(0.0, 0.0),
         plt_size:       vec2(0.0, 0.0),
         yaxis_on:       false,
         show_chans:     vec![vec![], vec![]],
         chan_ctrl:      ChanInfo::default(),
         num_chans:      0,
         adjust_y:       false,
      }
   }
}



impl MyApp {

   pub fn new(cc: &eframe::CreationContext) -> Self {
      cc.egui_ctx.set_visuals(egui::Visuals::dark());
      configure_fonts(&cc.egui_ctx);
      board_shim::disable_board_logger().unwrap();
      let mut app: MyApp = Default::default();
      app.board.prepare_session().unwrap();
      app.board.start_stream(45000, "").unwrap();
      app.is_streaming = true;
      let selected_chans = board_shim::get_eeg_channels(BOARD, PRESET).unwrap();
      app.num_chans = selected_chans.len();
      app.chan_ctrl.collect();
      app.show_chans = selected_chans.chunks(COLS).map(|s| s.into()).collect();
      app
   }

   fn top_panel(&mut self, ctx: &Context) {
      TopBottomPanel::top("top_panel").show(ctx, |ui| {
         ui.horizontal(|ui| {
            if self.board.is_prepared().unwrap() {
               if button(ui, "Release Session", FS15, 30.).clicked() {
                  self.board.release_session().unwrap();
                  self.is_streaming = false;
               }
               if self.is_streaming {
                  if button(ui, "Stop Streaming", FS15, 30.).clicked() {
                     self.board.stop_stream().unwrap();
                     self.is_streaming = false;
                  }
               } else {
                  if button(ui, "Start Streaming", FS15, 30.).clicked() {
                     self.board.start_stream(45000, "").unwrap();
                     self.is_streaming = true;
                  }
               }
            } else {
               if button(ui, "Prepare Session", FS15, 30.).clicked() {
                  self.board.prepare_session().unwrap();
               }
            }
            ui.add_space(50.);
            if self.is_streaming {
               let title = if self.yaxis_on { "Hide y-axis" } else { "Show y-axis" };
               if button(ui, title, FS15, 10.0).clicked() {
                  self.yaxis_on = !self.yaxis_on;
               }
               if button(ui, "Fit y", FS15, 10.0).clicked() {
                  self.adjust_y = true;
               }
            }
         });
      });
   }



   fn choose_chans(&mut self, ui: &mut Ui) -> (Vec<Vec<usize>>, usize) {
      ui.add_space(32.0);
      ui.label(RichText::new("Board Description:").font(FontId::proportional(FS16)).color(Color32::GREEN));
      ui.add_space(10.0);
      ui.label(RichText::new(self.chan_ctrl.description.as_str()).font(FontId::proportional(FS16)));      
      ui.add_space(32.0);
      ui.label(RichText::new("Select Channels:").font(FontId::proportional(FS16)).color(Color32::GREEN));
      ui.add_space(20.0);
      ui.horizontal(|ui| {
         ui.add_space(10.0);
         ui.checkbox(
            &mut self.chan_ctrl.eeg_checked,
            RichText::new("EEG Channels").font(FontId::proportional(FS16))
         );
      });
      ui.horizontal(|ui| {
         ui.add_space(10.0);
         ui.checkbox(
            &mut self.chan_ctrl.marker_checked,
            RichText::new("Marker Channels").font(FontId::proportional(FS16))
         );
      });
      ui.horizontal(|ui| {
         ui.add_space(10.0);
         ui.checkbox(
            &mut self.chan_ctrl.battery_checked,
            RichText::new("Battery Channels").font(FontId::proportional(FS16))
         );
      });
      let mut channels: HashSet<usize> = HashSet::new();
      if self.chan_ctrl.eeg_checked {
         let eeg: HashSet<usize> = self.chan_ctrl.eeg_indices.iter().copied().collect();
         channels.extend(&eeg);
      }

      if self.chan_ctrl.marker_checked {
         channels.insert(self.chan_ctrl.marker_index);
      }

      if self.chan_ctrl.battery_checked {
         channels.insert(self.chan_ctrl.battery_index);
      }
      let mut vector = channels.into_iter().collect::<Vec<_>>();
      vector.sort();

      let stuff_str: String = vector.iter().map( |&id| id.to_string() + ", ").collect();
      ui.add_space(10.0);
      ui.label(RichText::new(stuff_str).font(FontId::proportional(FS16)).color(Color32::GREEN));
      let n = vector.len();
      let out: Vec<Vec<usize>> = vector.chunks(COLS).map(|s| s.into()).collect();
      (out, n)
   }

   fn plt_data(&mut self, ui: &mut Ui, data: &Array2<f64>) {
      for list in self.show_chans.iter() {
         for chn in list {
            let row = data.row(*chn).iter().enumerate().map(|(i, y)| {Pos2 {x: i as f32, y: *y as f32 }}).collect();
            let memory = self.hash_map.entry(format!("p{chn}")).or_default();
            let mut plt: Plot = Plot::new(memory, self.plt_size, self.samples, self.yaxis_on);
            if self.adjust_y {
               let (min, max) = min_max_finder(&row);
               plt.set_y_range(min, max);
            }
            plt.show(ui, &row, *chn);
         }
         ui.end_row();
      }
   }

   fn plot_xaxis(&mut self, ui: &mut Ui, s: Vec2) {
      Resize::default().id_source(format!("{}", 1)).fixed_size(s).show(ui, |ui: &mut Ui| {
         let (response, painter) = ui.allocate_painter(s, Sense::drag() ); //mut painter
         let xaxis_rect = Rect::from_min_max(
            response.rect.min + vec2(if self.yaxis_on { 40.0 } else { 0.0 }, 0.0),
            response.rect.max,
         );
         //painter.set_clip_rect(response.rect);
         let ticks = calc_ticks(0.0, self.samples as f32);
         for tick_x in ticks {
            let tk = Pos2 {
               x: remap(tick_x, 0.0..=self.samples as f32, xaxis_rect.x_range()),
               y: xaxis_rect.min.y,
            };
            let x_points = [tk, tk+5.0*Vec2::Y];
            painter.line_segment(x_points, Stroke::new(1.0, Color32::WHITE));
            painter.line_segment([Pos2{x:0.0,y:0.0}, Pos2{x:20., y:20.}], Stroke::new(1.0, Color32::WHITE));
            let font_id = FontId{size: 10., family: FontFamily::Proportional};
            painter.text(tk+15.0*Vec2::Y, CCENTER, tick_x, font_id, Color32::WHITE );
         }
      });
   }

   fn key_events(&mut self, ctx: &Context) {
      let keys_down: HashSet<Key> = ctx.input(|i|
         i.keys_down.to_owned()
      );
      if keys_down.contains(&Key::ArrowUp) {
         self.samples += 10;
         if self.samples > 2000 { self.samples = 2000; }
      } else if keys_down.contains(&Key::ArrowDown) {
         self.samples -= 10;
         if self.samples < 100 { self.samples = 100; }
      } else if keys_down.contains(&Key::Escape) {
         ctx.send_viewport_cmd(ViewportCommand::Close);
      }
   }

   fn update_plt_size(&mut self, s: Vec2, n: usize) {
      let rows = n.div_ceil(COLS) as f32;
      self.plt_size = vec2(
         ((s.x-20.0) / (COLS as f32)).round(),
         ((s.y-20.0-20.0) / rows).round()
      );
      self.ui_size = s;
      self.num_chans = n;
   }
}


impl eframe::App for MyApp {
   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
      ctx.request_repaint();
      self.top_panel(ctx);
      self.key_events(ctx);
      CentralPanel::default().show(ctx, |ui| {
         let s: Vec2 = ui.available_size();
         if !s.eq(&self.ui_size) {
            self.update_plt_size(s, self.num_chans);
         }
         if !self.is_streaming {
            let (vec, n) = self.choose_chans(ui);
            if vec != self.show_chans {
               self.show_chans = vec;
               self.update_plt_size(s, n);
            }
            return;
         }
         Grid::new("grid").show(ui, |ui| {
            self.plt_data(ui, &self.board.get_current_board_data(self.samples, PRESET).unwrap());
         });
         ui.horizontal(|ui| {
            for _ in 0..COLS { self.plot_xaxis(ui, self.plt_size); }
         });
         self.adjust_y = false;
      });
   }

   fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
      if self.board.is_prepared().unwrap() {
         self.board.release_session().unwrap();
      }
   }
}


fn button(ui: &mut Ui, text: &str, size: f32, space: f32) -> Response {
   let btn: Response = ui.add_sized(
      Vec2 {x: 130.0, y: 30.0 },
      Button::new(RichText::new(text).font(FontId::proportional(size))).frame(false)
   );
   ui.add_space(space);
   btn
}


pub fn configure_fonts(ctx: &Context) {
   let mut fonts = FontDefinitions::default();
   fonts.font_data.insert(
      "MesloLGS".to_owned(),
      //FontData::from_static(include_bytes!("../fonts/MesloLGS_NF_Regular.ttf"))
        Arc::new(FontData::from_static(include_bytes!(
            "../fonts/MesloLGS_NF_Regular.ttf"
        ))),
   );
   fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "MesloLGS".to_owned());
   fonts.families.get_mut(&FontFamily::Monospace).unwrap().push("MesloLGS".to_owned());
   ctx.set_fonts(fonts);
}

/*
fn min_max_finder(numbers: &Vec<f32>)-> (f32, f32) {
   let mut max: f32 = numbers[0];
   let mut min: f32 = numbers[0];
   for i in numbers.iter() {
      if i > &max {
         max = *i
      }
      if i < &min {
         min = *i
      }
   }
   (min, max)
}*/

fn min_max_finder(pos_list: &Vec<Pos2>)-> (f32, f32) {
   let mut max: f32 = pos_list[0].y;
   let mut min: f32 = pos_list[0].y;
   for item in pos_list.iter() {
      if item.y > max {
         max = item.y
      }
      if item.y < min {
         min = item.y
      }
   }
   if min == max {
      min -= 1.0;
      max += 1.0;
   } else {
      let e = (max - min) *0.05;
      min -= e;
      max += e;
   }
   (min.floor(), max.ceil())
}

// https://stackoverflow.com/questions/237220/tickmark-algorithm-for-a-graph-axis
fn calc_ticks(start: f32, end: f32) -> Vec<f32> {
   let range = end - start;
   let x: f32 = 10.0_f32.powf(range.log10().floor());
   let range_over_x = range / x;
   let step = if range_over_x >= 5.0 {
      2.0 * x
   } else if range_over_x >= 2.5 {
        x
   } else {
        x / 2.5
   };
   let mut ticks: Vec<f32> = vec![];
   let mut i: f32 = start;
   loop {
      ticks.push(i);
      i = i + step;
      if i > end { break;}
   }
   ticks
}
