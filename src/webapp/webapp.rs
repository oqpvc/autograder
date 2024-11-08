use eframe::egui;
use eframe::egui::Context;

use crate::webapp::create_key::CreateKey;
use crate::webapp::create_template::CreateTemplate;

use crate::webapp::generate_report::GenerateReport;
use crate::webapp::help::Help;

pub struct WebApp {
    current_view: ViewType,
    generate_report: GenerateReport,
    create_template: CreateTemplate,
    create_key: CreateKey,
    help: Help,
}

enum ViewType {
    GenerateReport,
    CreateTemplate,
    CreateKey,
    Help,
}

impl Default for WebApp {
    fn default() -> Self {
        Self {
            current_view: ViewType::Help,
            generate_report: GenerateReport::default(),
            create_template: CreateTemplate::default(),
            create_key: CreateKey::default(),
            help: Help::default(),
        }
    }
}

impl eframe::App for WebApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Navigation bar
        egui::TopBottomPanel::top("nav_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Generate Report").clicked() {
                    self.current_view = ViewType::GenerateReport;
                }
                if ui.button("Create Template").clicked() {
                    self.current_view = ViewType::CreateTemplate;
                }
                if ui.button("Create Key").clicked() {
                    self.current_view = ViewType::CreateKey;
                }
                if ui.button("Help").clicked() {
                    self.current_view = ViewType::Help;
                }
            });
        });

        match self.current_view {
            ViewType::GenerateReport => self.generate_report.update(ctx, frame),
            ViewType::CreateTemplate => self.create_template.update(ctx, frame),
            ViewType::CreateKey => self.create_key.update(ctx, frame),
            ViewType::Help => self.help.update(ctx, frame),
        }
    }
}
