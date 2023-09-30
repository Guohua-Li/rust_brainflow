// Codes in this file were modified after: https://github.com/EmbersArc/eplot/tree/master

use egui::{
    Ui, Id, 
    FontId,
    FontFamily,
    Color32,
    Align2,
    CursorIcon,
    Vec2,
    vec2,
    Pos2,
    pos2,
    Shape,
    remap,
    Stroke,
    Resize,
    Rect,
    Sense
};

use std::collections::HashMap;


const GREEN: Color32 = Color32::from_rgb(0, 255, 0);
const WHITE: Color32 = Color32::from_rgb(200, 200, 200);
const GRAY10: Color32 = Color32::from_gray(10);
const GRAY50: Color32 = Color32::from_gray(50);
const GRAY150: Color32 = Color32::from_gray(150);
const MARGIN_LEFT: f32 = 40.0;
const SPACINGPLOTS: f32 = 4.0;



#[derive(Default)]
pub struct PlotCtx {
    pub(crate) hash_map: HashMap<Id, PlotMemory>,
}

impl PlotCtx {
    pub fn plot(&mut self, label: impl Into<String>) -> Plot {
        let id: Id = Id::new(label.into());
        let memory: &mut PlotMemory = self.hash_map.entry(id).or_default();
        Plot::new_with_memory(memory)
    }
}


pub(crate) struct PlotMemory {
    m_pos_last: Option<Pos2>,
    m_yo: f32,
    m_ye: f32,
}


impl Default for PlotMemory {
    fn default() -> Self {
        Self {
            m_pos_last: None,
            m_yo: -300.0,
            m_ye: 300.0,
        }
    }
}


pub struct Plot<'a> {
    plt_memory: &'a mut PlotMemory,
    size: Vec2,
    title: Option<String>,
    show_val_at_cursor: bool,
    show_title: bool,
    show_xticks: bool,
    show_yticks: bool,
    drag_yaxis: bool,
    zoom_y: bool,
    x_label: String,
    x_end: f32,
    y_start: f32,
    y_end: f32,
}

impl<'a> Plot<'a> {
    fn new_with_memory(plt_memory: &'a mut PlotMemory) -> Self {
        Self {
            plt_memory,
            size: vec2(100., 100.),
            title: None,
            show_val_at_cursor: false,
            show_title: false,
            show_xticks: false,
            show_yticks: false,
            drag_yaxis: false,
            zoom_y: false,
            x_label: "".to_string(),
            x_end: 1000.0,
            y_start: -350.0,
            y_end: 350.0,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn show_title(mut self, show: bool) -> Self {
        self.show_title = show;
        self
    }

    pub fn show_xlabel(mut self, show: bool) -> Self {
        self.show_xticks = show;
        self
    }

    pub fn show_ytick_val(mut self, show: bool) -> Self {
        self.show_yticks = show;
        self
    }

    pub fn set_drag_yaxis(mut self, drag: bool) -> Self {
        self.drag_yaxis = drag;
        self
    }

    pub fn set_zoom_y(mut self, z: bool) -> Self {
        self.zoom_y = z;
        self
    }

    pub fn show_xtick_val(mut self, show: bool) -> Self {
        self.show_xticks = show;
        self
    }

    pub fn set_x_end(mut self, e: f32) -> Self {
        self.x_end = e;
        self
    }

    pub fn set_y_end(mut self, e: f32) -> Self {
        self.y_end = e;
        self
    }

    pub fn show(self, ui: &mut Ui, raw_data: Vec<Pos2>) {//mut 
        let Self {
            show_val_at_cursor,
            plt_memory,
            title,
            size,
            show_title,
            show_xticks,
            show_yticks,
            drag_yaxis,
            zoom_y,
            mut y_start,
            mut y_end,
            ..
        } = self;

        let id_string = format!("{}", "resize".to_string());
        let resize_area = Resize::default()
            .id_source(id_string)
            .fixed_size(size);//max_size(size).min_size(size*0.85) auto_sized()

        resize_area.show(ui, |ui: &mut Ui| {
            let PlotMemory {
                m_pos_last,
                m_yo,
                m_ye,
            } = plt_memory;

            y_start = *m_yo;
            y_end   = *m_ye;

            let space_left: Vec2 = ui.available_size_before_wrap();
            let (response, mut painter) = ui.allocate_painter(space_left, Sense::drag());
            let left_margin = { // mut
                if show_yticks { MARGIN_LEFT } else { 0.0 }
            };

            //if !x_axis.label.is_empty() { bottom_margin += 10.0 }
            let bottom_margin = {
                if show_xticks { 20.0 } else { 0.0 }
            };

            let top_margin = {
                if show_title { 10.0 } else { 0.0 } // we never show the title
            };

            let full_rect = response.rect;

            let plot_rect = Rect::from_min_max(
                full_rect.min + vec2(left_margin, top_margin),
                full_rect.max - vec2(SPACINGPLOTS, bottom_margin),
            );

            painter.rect(plot_rect, 0.0, GRAY10, Stroke::new(1.0, GRAY150)); // plot the frame

            if show_title {
                if let Some(ref t) = title { // we do have a title
                    let pos = plot_rect.center_top() - vec2(0.0, 2.0);
                    let font_id = FontId{size: 15., family:FontFamily::Proportional};
                    painter.text(pos, Align2::CENTER_BOTTOM, t, font_id, WHITE);
                }
            }

            if !self.x_label.is_empty() { // currently we don't have a x-axis label
                let pos = plot_rect.center_bottom() + vec2(0.0, 25.0);
                let font_id = FontId{size: 15., family:FontFamily::Proportional};
                painter.text(pos, Align2::CENTER_TOP, self.x_label.clone(), font_id, WHITE);
            }

            // Dragging
            if drag_yaxis {
                let drag_pos: Option<Pos2> = response.interact_pointer_pos();
                if let Some(pos) = drag_pos {
                    let x_tf = remap(pos.x, plot_rect.x_range(), 0.0..=self.x_end);//self.
                    let y_tf = remap(pos.y, plot_rect.y_range(), y_end..=y_start);
                    if let Some(p0) = m_pos_last {
                        ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);
                        let x0_tf = remap(p0.x, plot_rect.x_range(), 0.0..=self.x_end);//self.
                        let y0_tf = remap(p0.y, plot_rect.y_range(), y_end..=y_start);
                        let delta = Pos2::new(x0_tf, y0_tf) - Pos2::new(x_tf, y_tf);
                        y_start += delta.y;
                        y_end   += delta.y;
                    }
                    *m_pos_last = Some(pos);
                } else {
                    *m_pos_last = None;
                }
            }

            // Zooming
            if zoom_y {
                let scrolled = ui.input(|i| i.scroll_delta.y.clamp(-10.0, 10.0));
                let pointer_pos = ui.input(|i| i.pointer.interact_pos());
                if let Some(mouse_pos) = pointer_pos.filter(|pos| plot_rect.contains(*pos)) {
                    if scrolled != 0.0 {
                        let zoom_factor = -0.01 * scrolled;
                        let center: f32 = (plot_rect.bottom() - mouse_pos.y) / plot_rect.height();
                        let ext = y_end - y_start;
                        y_start -= zoom_factor * center * ext;
                        y_end   += zoom_factor * (1.0 - center) * ext;
                    }
                }
            }

            let plt_to_scr = |plt_pos: &Pos2| -> Pos2 {
                let x_tf = remap(plt_pos.x, 0.0..=self.x_end, plot_rect.x_range().clone());//self.
                let y_tf = remap(plt_pos.y, y_end..=y_start, plot_rect.y_range().clone());
                pos2(x_tf, y_tf)
            };

            let scr_to_plt = |pix_pos: &Pos2| -> Pos2 {
                let x_tf = remap(pix_pos.x, plot_rect.x_range().clone(), 0.0..=self.x_end);//self.
                let y_tf = remap(pix_pos.y, plot_rect.y_range().clone(), y_end..=y_start);
                pos2(x_tf, y_tf)
            };

            // Ticks and tick labels
            let rough_inc = self.x_end.min(y_end-y_start) / 5.0; // The lower limit
            let inc = emath::smart_aim::best_in_range_f64( (rough_inc*0.5) as f64, (rough_inc*1.5) as f64 ) as f32;

            // X-Axis ticks
            let mut i_start = (0.0 / inc) as i32;//x_start
            if i_start >= 0 { i_start += 1; }
            loop {
                let tick_x = i_start as f32 * inc;
                if tick_x > self.x_end { break; }
                let tk = plt_to_scr(&pos2(tick_x, y_start));
                let v_points = [tk, tk-plot_rect.height()*Vec2::Y];
                painter.line_segment(v_points, Stroke::new(0.5, GRAY50) );
                if show_xticks {
                    let x_points = [tk, tk-5.0*Vec2::Y];
                    painter.line_segment(x_points, Stroke::new(1.0, WHITE));
                    let font_id = FontId{size: 11., family:FontFamily::Proportional};
                    painter.text(tk+15.0*Vec2::Y, Align2::CENTER_CENTER, tick_x, font_id, WHITE );
                }
                i_start += 1;
            }

            // Y-Axis ticks
            let mut i_start = (y_start / inc) as i32;
            if i_start >= 0 { i_start += 1; }
            loop {
                let tick_y = i_start as f32 * inc;
                if tick_y > y_end { break; }
                let tk = plt_to_scr(&pos2(0.0, tick_y));//x_start
                let h_points = [tk, tk + plot_rect.width() * Vec2::X];
                painter.line_segment(h_points, Stroke::new(0.5, GRAY50) );
                if show_yticks {
                    let y_points = [tk, tk + 5.0 * Vec2::X];
                    painter.line_segment(y_points, Stroke::new(1.0, WHITE) );
                    let font_id = FontId{size: 11., family:FontFamily::Proportional};
                    painter.text(tk - 15.0 * Vec2::X, Align2::RIGHT_CENTER, tick_y, font_id, WHITE );
                }
                i_start += 1;
            }

            *m_yo = y_start;
            *m_ye = y_end;

            painter.set_clip_rect(plot_rect);

            // ---------- ready to plot ----------
            let points: Vec<Pos2> = raw_data.iter().map(|p| plt_to_scr(p)).collect();
            painter.add(Shape::line(points, Stroke::new(1.0, GREEN)));

            if show_val_at_cursor {
                let pointer_pos = ui.input(|i| i.pointer.interact_pos());
                if let Some(mouse_pos) = pointer_pos.filter(|pos| plot_rect.contains(*pos)) {
                    let mouse_pos = scr_to_plt(&mouse_pos);
                    let txt_pos = plot_rect.right_bottom() + vec2(-10.0, -10.0);
                    let font_id = FontId{size: 15., family:FontFamily::Proportional};
                    painter.text(txt_pos, Align2::RIGHT_BOTTOM, format!("{:?}", mouse_pos), font_id, WHITE );
                }
            }
        })
    }
}
