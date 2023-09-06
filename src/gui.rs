use eframe::egui::{CentralPanel, Slider};

use crate::App;

pub struct Gui {
    pub app: App,
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
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
        });
    }
}
