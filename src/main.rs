use std::path::PathBuf;
use std::process::{Child, Command};

use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
enum DisplayKind {
    Gtk,
    Sdl,
    None,
}

impl DisplayKind {
    fn as_str(&self) -> &'static str {
        match self {
            DisplayKind::Gtk => "gtk",
            DisplayKind::Sdl => "sdl",
            DisplayKind::None => "none",
        }
    }
}

impl std::str::FromStr for DisplayKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gtk" => Ok(DisplayKind::Gtk),
            "sdl" => Ok(DisplayKind::Sdl),
            "none" => Ok(DisplayKind::None),
            _ => Err(format!("unknown display: {s}")),
        }
    }
}

impl Serialize for DisplayKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DisplayKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    qemu_path: String,
    disk_path: String,
    iso_path: String,
    ram_mb: u32,
    smp: u32,
    display: DisplayKind,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            qemu_path: r"D:\qemu\qemu-system-x86_64.exe".into(),
            disk_path: r"D:\NixOS\..\qemu\vms\nixos\nixos-disk.qcow2".into(),
            iso_path: r"D:\NixOS\..\qemu\vms\nixos\nixos-minimal-26.iso".into(),
            ram_mb: 4096,
            smp: 2,
            display: DisplayKind::Gtk,
        }
    }
}

struct QemuGui {
    config: Config,
    config_path: PathBuf,
    log: String,
    child: Option<Child>,
}

impl Default for QemuGui {
    fn default() -> Self {
        let config_path = PathBuf::from("qemu-gui-config.ron");

        let config = if config_path.exists() {
            std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|s| ron::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Config::default()
        };

        Self {
            config,
            config_path,
            log: String::new(),
            child: None,
        }
    }
}

impl QemuGui {
    fn save_config(&mut self) {
        match ron::to_string(&self.config) {
            Ok(s) => match std::fs::write(&self.config_path, s) {
                Ok(_) => self.log_push("Config saved.\n"),
                Err(e) => self.log_push(&format!("Failed to write config: {e}\n")),
            },
            Err(e) => self.log_push(&format!("Failed to serialize config: {e}\n")),
        }
    }

    fn log_push(&mut self, msg: &str) {
        const MAX_LOG: usize = 10_240;
        if self.log.len() + msg.len() > MAX_LOG {
            let remove = self.log.len() + msg.len() - MAX_LOG;
            self.log.drain(..remove);
        }
        self.log.push_str(msg);
    }

    fn validate(&self, boot_from_cd: bool) -> Result<(), String> {
        if self.config.qemu_path.trim().is_empty() {
            return Err("QEMU path is empty".into());
        }
        if self.config.disk_path.trim().is_empty() {
            return Err("Disk image path is empty".into());
        }
        if boot_from_cd && self.config.iso_path.trim().is_empty() {
            return Err("ISO path is empty".into());
        }
        if self.config.ram_mb == 0 {
            return Err("RAM must be > 0".into());
        }
        if self.config.smp == 0 {
            return Err("SMP must be > 0".into());
        }
        Ok(())
    }

    fn build_args(&self, boot_from_cd: bool) -> Vec<String> {
        let mut args = Vec::new();

        args.push("-machine".to_string());
        args.push("pc,accel=whpx,kernel-irqchip=off".to_string());

        args.push("-m".to_string());
        args.push(self.config.ram_mb.to_string());

        args.push("-smp".to_string());
        args.push(self.config.smp.to_string());

        args.push("-drive".to_string());
        args.push(format!(
            "file={},if=virtio,format=qcow2",
            self.config.disk_path
        ));

        if boot_from_cd {
            args.push("-cdrom".to_string());
            args.push(self.config.iso_path.clone());
            args.push("-boot".to_string());
            args.push("d".to_string());
        } else {
            args.push("-boot".to_string());
            args.push("c".to_string());
        }

        args.push("-netdev".to_string());
        args.push("user,id=net0".to_string());
        args.push("-device".to_string());
        args.push("virtio-net,netdev=net0".to_string());

        args.push("-vga".to_string());
        args.push("std".to_string());

        args.push("-display".to_string());
        args.push(self.config.display.as_str().to_owned());

        args.push("-usb".to_string());
        args.push("-device".to_string());
        args.push("usb-tablet".to_string());

        args
    }

    fn launch_qemu(&mut self, boot_from_cd: bool) {
        if let Err(e) = self.validate(boot_from_cd) {
            self.log_push(&format!("Validation failed: {e}\n"));
            return;
        }

        let args = self.build_args(boot_from_cd);

        self.log_push(&format!(
            "Launching: {} {}\n",
            self.config.qemu_path,
            args.join(" ")
        ));

        match Command::new(&self.config.qemu_path).args(&args).spawn() {
            Ok(child) => {
                self.child = Some(child);
                self.log_push("QEMU started.\n");
            }
            Err(e) => {
                self.log_push(&format!("Failed to launch QEMU: {e}\n"));
            }
        }
    }

    fn kill_qemu(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            self.log_push("QEMU terminated.\n");
        }
    }
}

impl Drop for QemuGui {
    fn drop(&mut self) {
        self.save_config();
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
    }
}

impl eframe::App for QemuGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("QEMU GUI Wrapper");

                ui.separator();
                ui.label("QEMU Path:");
                ui.text_edit_singleline(&mut self.config.qemu_path);

                ui.label("Disk Image:");
                ui.text_edit_singleline(&mut self.config.disk_path);

                ui.horizontal(|ui| {
                    ui.label("ISO Path:");
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("ISO", &["iso"])
                            .set_title("Select ISO File")
                            .pick_file()
                        {
                            self.config.iso_path = path.display().to_string();
                        }
                    }
                });
                ui.text_edit_singleline(&mut self.config.iso_path);

                ui.horizontal(|ui| {
                    ui.label("RAM (MB):");
                    ui.add(egui::Slider::new(&mut self.config.ram_mb, 512..=32768));
                });

                ui.horizontal(|ui| {
                    ui.label("SMP (CPU cores):");
                    ui.add(egui::Slider::new(&mut self.config.smp, 1..=32));
                });

                ui.horizontal(|ui| {
                    ui.label("Display:");
                    egui::ComboBox::from_id_salt("display")
                        .selected_text(self.config.display.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.config.display, DisplayKind::Gtk, "GTK");
                            ui.selectable_value(&mut self.config.display, DisplayKind::Sdl, "SDL");
                            ui.selectable_value(
                                &mut self.config.display,
                                DisplayKind::None,
                                "None",
                            );
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(format!("Config: {}", self.config_path.display()));
                    if ui.button("Save Config").clicked() {
                        self.save_config();
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Boot from Disk").clicked() {
                        self.launch_qemu(false);
                    }
                    if ui.button("Install (Boot from ISO)").clicked() {
                        self.launch_qemu(true);
                    }
                    if ui.button("Kill QEMU").clicked() {
                        self.kill_qemu();
                    }
                });

                ui.separator();
                ui.label("Log:");
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.log)
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(10)
                                .desired_width(f32::INFINITY),
                        );
                    });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([700.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "QEMU GUI",
        options,
        Box::new(|_cc| Ok(Box::new(QemuGui::default()))),
    )
}
