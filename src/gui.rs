use std::{ops::RangeInclusive, time::Duration};

use eframe::egui::{
    plot::{Legend, Plot, Points},
    CentralPanel, Slider,
};

use crate::App;

pub struct Gui {
    pub app: App,
}

impl App {
    pub fn slider<'a>(
        data: &'a mut f32,
        range: RangeInclusive<f32>,
        text: &'static str,
    ) -> Slider<'a> {
        Slider::new(data, range)
            .text(text)
            .orientation(eframe::egui::SliderOrientation::Vertical)
            .trailing_fill(true)
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            eframe::egui::global_dark_light_mode_switch(ui);
            ui.horizontal(|ui| {
                ui.add(App::slider(self.app.k_p.as_mut(), 0.0..=50.0, "P gain"));
                ui.add(App::slider(self.app.k_i.as_mut(), 0.0..=50.0, "I gain"));
                ui.add(App::slider(self.app.k_d.as_mut(), 0.0..=50.0, "D gain"));
                ui.add(App::slider(
                    self.app.motor_gain.as_mut(),
                    0.0..=10.0,
                    "Motor  Gain",
                ));
            });

            let plot = Plot::new("err_demo")
                .legend(Legend::default())
                .data_aspect(1.0);

            plot.show(ui, |ui| {
                let len = self.app.err_linear.len();
                if len > 10 {
                    let points = self
                        .app
                        .err_linear
                        .iter()
                        .enumerate()
                        .map(|(index, &item)| [index as f64 / 100.0, item])
                        .collect::<Vec<_>>();
                    let points = Points::new(points)
                        .name("Error Linear")
                        .stems(0.0)
                        .radius(1.0);
                    ui.points(points)
                }

                let len = self.app.err_angular.len();
                if len > 10 {
                    let points = self
                        .app
                        .err_angular
                        .iter()
                        .enumerate()
                        .map(|(index, &item)| [index as f64 / 100.0, item])
                        .collect::<Vec<_>>();
                    let points = Points::new(points)
                        .name("Error Angular")
                        .stems(0.0)
                        .radius(1.0);
                    ui.points(points)
                }
            });
        });
        ctx.request_repaint_after(Duration::from_secs((1. / 60.) as u64));
    }
}
