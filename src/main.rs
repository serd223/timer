#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::time::{Duration, Instant};

use eframe::{
    egui::{self, TextEdit},
    CreationContext,
};

// 227x121
const DEFAULT_WIDTH: f32 = 227.;
const DEFAULT_HEIGHT: f32 = 121.;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(DEFAULT_WIDTH, DEFAULT_HEIGHT)),
        ..Default::default()
    };

    eframe::run_native("Timer", options, Box::new(|cc| Box::new(App::from_cc(cc))))
}

#[derive(Clone)]
struct DateStr {
    hour: String,
    minute: String,
    second: String,
}

impl Default for DateStr {
    fn default() -> Self {
        Self {
            hour: String::from("00"),
            minute: String::from("00"),
            second: String::from("00"),
        }
    }
}

impl DateStr {
    fn from_seconds(sec: u64) -> Self {
        let h = sec / 3600;
        let m = (sec % 3600) / 60;
        let s = sec % 60;

        let hour = format!("{h:0>2}");
        let minute = format!("{m:0>2}");
        let second = format!("{s:0>2}");

        Self {
            hour,
            minute,
            second,
        }
    }
    fn parse_secs(&self) -> Result<u64, std::num::ParseIntError> {
        let mut res = 0;

        res += self.hour.parse::<u64>().unwrap_or(0) * 3600;
        res += self.minute.parse::<u64>().unwrap_or(0) * 60;
        res += self.second.parse::<u64>().unwrap_or(0);

        Ok(res)
    }
}

struct App {
    input: DateStr,
    marked: DateStr,
    target: Instant,
    paused: bool,
    remaining: u64,
    timer_duration: Duration,
}

impl App {
    fn from_cc(cc: &CreationContext) -> Self {
        match cc.storage {
            None => Self::default(),
            Some(storage) => {
                let remaining = storage.get_string("remaining");
                let duration = storage.get_string("duration");
                let marked = storage.get_string("marked");
                match (duration, remaining, marked) {
                    (Some(duration), Some(remaining), Some(marked)) => {
                        let remaining = remaining.parse::<u64>().unwrap();
                        let duration = duration.parse::<u64>().unwrap();
                        let marked = marked.parse::<u64>().unwrap();
                        App {
                            target: Instant::now()
                                .checked_add(Duration::from_secs(duration))
                                .unwrap(),
                            remaining,
                            timer_duration: Duration::from_secs(duration),
                            input: DateStr::from_seconds(remaining),
                            marked: DateStr::from_seconds(marked),
                            ..Default::default()
                        }
                    }
                    _ => App::default(),
                }
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            input: DateStr::default(),
            marked: DateStr::default(),
            target: Instant::now(),
            paused: true,
            remaining: 0,
            timer_duration: Duration::from_secs(0),
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("remaining", self.remaining.to_string());
        storage.set_string("duration", self.timer_duration.as_secs().to_string());
        storage.set_string("marked", self.marked.parse_secs().unwrap().to_string());
        storage.flush();
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let eframe::epaint::Vec2 {
            x: width,
            y: height,
        } = frame.info().window_info.size;
        ctx.request_repaint_after(Duration::from_secs(1));
        egui::CentralPanel::default().show(ctx, |ui| {
            let space_pressed = ctx.input(|k| k.key_pressed(egui::Key::Space));

            ui.vertical_centered(|ui| {
                if width == 0. || height == 0. {
                    return;
                }
                for (_, font_id) in ui.style_mut().text_styles.iter_mut() {
                    font_id.size *= (width * height).sqrt()
                        / ((DEFAULT_WIDTH * DEFAULT_HEIGHT) as f32).sqrt()
                        * 1.85;

                    // * 2.5;
                    // font_id.size *= 1.85;
                }

                ui.add_space(height / 15.);
                ui.columns(5, |columns| {
                    let te = TextEdit::singleline(&mut self.input.hour)
                        .char_limit(2)
                        .horizontal_align(egui::Align::Center);
                    columns[1].add_enabled(self.paused, te);

                    let te = TextEdit::singleline(&mut self.input.minute)
                        .char_limit(2)
                        .horizontal_align(egui::Align::Center);
                    columns[2].add_enabled(self.paused, te);

                    let te = TextEdit::singleline(&mut self.input.second)
                        .char_limit(2)
                        .horizontal_align(egui::Align::Center);
                    columns[3].add_enabled(self.paused, te);
                });

                if self.paused {
                    if ui.button("Start").clicked() || space_pressed {
                        self.input.hour = format!("{:0>2}", self.input.hour);
                        self.input.minute = format!("{:0>2}", self.input.minute);
                        self.input.second = format!("{:0>2}", self.input.second);
                        match self.input.parse_secs() {
                            Ok(s) => {
                                self.paused = false;
                                self.target =
                                    Instant::now().checked_add(Duration::from_secs(s)).unwrap();
                                self.timer_duration = Duration::from_secs(s);
                                self.remaining = self.timer_duration.as_secs();
                            }

                            Err(_) => (),
                        }
                    }
                } else {
                    if ui.button("Pause").clicked() || space_pressed {
                        self.paused = true;
                    }
                }

                let now = Instant::now();

                if self.paused {
                    self.target = now
                        .checked_add(Duration::from_secs(self.remaining))
                        .unwrap();
                }

                // if ui.button("Restart").clicked() {
                //     self.target = now.checked_add(self.timer_duration).unwrap();
                // }

                if now < self.target {
                    self.remaining = (self.target - now).as_secs();
                    let h = self.remaining / 3600;
                    let m = (self.remaining % 3600) / 60;
                    let s = self.remaining % 60;

                    if !self.paused {
                        self.input.hour = format!("{h:0>2}");
                        self.input.minute = format!("{m:0>2}");
                        self.input.second = format!("{s:0>2}");
                    }
                }

                if ui
                    .button(
                        format!(
                            "Mark: {}:{}:{}",
                            self.marked.hour, self.marked.minute, self.marked.second
                        )
                        .as_str(),
                    )
                    .clicked()
                {
                    self.marked = self.input.clone();
                }
            });
        });
    }
}
