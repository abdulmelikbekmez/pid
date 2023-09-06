use std::time::Duration;

use eframe::egui::{
    plot::{Legend, Plot, Points},
    CentralPanel, Slider,
};

use crate::App;

pub struct Gui {
    pub app: App,
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            eframe::egui::global_dark_light_mode_switch(ui);
            ui.horizontal(|ui| {
                ui.add(
                    Slider::new(self.app.k_p.tmp(), 0.0..=50.0)
                        .text("P gain")
                        .orientation(eframe::egui::SliderOrientation::Vertical),
                );

                ui.add(
                    Slider::new(self.app.k_i.tmp(), 0.0..=50.0)
                        .text("I gain")
                        .orientation(eframe::egui::SliderOrientation::Vertical),
                );

                ui.add(
                    Slider::new(self.app.k_d.tmp(), 0.0..=50.0)
                        .text("D gain")
                        .orientation(eframe::egui::SliderOrientation::Vertical),
                );
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
                    let points = Points::new(points).name("Error Linear");
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
                    let points = Points::new(points).name("Error Angular");
                    ui.points(points)
                }
            });
        });
        ctx.request_repaint_after(Duration::from_secs((1. / 60.) as u64));
    }
}
