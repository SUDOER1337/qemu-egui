use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

use eframe::egui;
use serde::{Deserialize, Serialize};

macro_rules! str_enum {
    ($vis:vis enum $name:ident { $($variant:ident = $str:expr),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $($variant),+
        }
        impl $name {
            fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $str),+
                }
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.as_str().serialize(serializer)
            }
        }
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let s = String::deserialize(deserializer)?;
                $(
                    if s.eq_ignore_ascii_case($str) {
                        return Ok(Self::$variant);
                    }
                )+
                Err(serde::de::Error::custom(format!("unknown {} variant: {s}", stringify!($name))))
            }
        }
    };
}

str_enum! {
    enum DisplayKind { Gtk = "gtk", Sdl = "sdl", Disabled = "none" }
}
str_enum! {
    enum MachineType { Pc = "pc", Q35 = "q35" }
}
str_enum! {
    enum VmPort { Auto = "auto", On = "on", Off = "off" }
}
str_enum! {
    enum OnOff { On = "on", Off = "off" }
}
str_enum! {
    enum AccelKind { Kvm = "kvm", Whpx = "whpx", Tcg = "tcg", Hax = "hax" }
}
str_enum! {
    enum IrqChip { Off = "off", Split = "split", On = "on" }
}
str_enum! {
    enum VgaKind { Std = "std", Virtio = "virtio", Qxl = "qxl", Vmware = "vmware", Cirrus = "cirrus", Disabled = "none" }
}
str_enum! {
    enum NicModel { VirtioNet = "virtio-net", E1000 = "e1000", Rtl8139 = "rtl8139", Ne2kPci = "ne2k_pci", Pcnet = "pcnet" }
}
str_enum! {
    enum NetBackend { User = "user", Tap = "tap" }
}
str_enum! {
    enum AudioDev { Disabled = "none", Pa = "pa", Alsa = "alsa", Dsound = "dsound" }
}
str_enum! {
    enum SoundHw { Disabled = "none", IntelHda = "intel-hda", Ac97 = "ac97", Sb16 = "sb16", Es1370 = "es1370" }
}
str_enum! {
    enum UsbDevice { Tablet = "usb-tablet", Mouse = "usb-mouse", Disabled = "none" }
}
str_enum! {
    enum DriveCache { Writeback = "writeback", Disabled = "none", Writethrough = "writethrough", Unsafe = "unsafe", Directsync = "directsync" }
}
str_enum! {
    enum DriveAio { Threads = "threads", Native = "native", IoUring = "io_uring" }
}
str_enum! {
    enum DriveIf { Virtio = "virtio", Ide = "ide", Sata = "sata", Sd = "sd" }
}
str_enum! {
    enum RtcBase { Utc = "utc", Localtime = "localtime" }
}
str_enum! {
    enum WatchdogKind { Off = "", I6300esb = "i6300esb", Ib700 = "ib700" }
}
str_enum! {
    enum WatchdogAction { Reset = "reset", Shutdown = "shutdown", Poweroff = "poweroff", InjectNmi = "inject-nmi", Disabled = "none", Pause = "pause", Debug = "debug" }
}
str_enum! {
    enum BootMenu { On = "on", Off = "off" }
}
str_enum! {
    enum BootStrict { On = "on", Off = "off" }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Config {
    qemu_path: String,
    disk_path: String,
    iso_path: String,
    ram_mb: u32,
    smp: u32,
    display: DisplayKind,
    boot_order: String,
    custom_args: String,
    machine_type: MachineType,
    cpu_model: String,
    accel: AccelKind,
    kernel_irqchip: IrqChip,
    vmport: VmPort,
    dump_guest_core: OnOff,
    vga: VgaKind,
    display_gl: OnOff,
    nic_model: NicModel,
    net_backend: NetBackend,
    audio_dev: AudioDev,
    sound_hw: SoundHw,
    usb_device: UsbDevice,
    boot_menu: BootMenu,
    boot_strict: BootStrict,
    drive_cache: DriveCache,
    drive_aio: DriveAio,
    drive_if: DriveIf,
    rtc_base: RtcBase,
    watchdog: WatchdogKind,
    watchdog_action: WatchdogAction,
    mem_slots: u32,
    mem_maxmb: u32,
    monitor_width: u32,
    monitor_height: u32,
}

impl Default for Config {
    fn default() -> Self {
        let (qemu_path, accel, audio_dev) = if cfg!(target_os = "linux") {
            (
                "/usr/bin/qemu-system-x86_64".into(),
                AccelKind::Kvm,
                AudioDev::Pa,
            )
        } else {
            (
                r"D:\qemu\qemu-system-x86_64.exe".into(),
                AccelKind::Whpx,
                AudioDev::Dsound,
            )
        };
        Self {
            qemu_path,
            disk_path: String::new(),
            iso_path: String::new(),
            ram_mb: 4096,
            smp: 2,
            display: DisplayKind::Gtk,
            boot_order: "c".into(),
            custom_args: String::new(),
            machine_type: MachineType::Pc,
            cpu_model: "host".into(),
            accel,
            kernel_irqchip: IrqChip::On,
            vmport: VmPort::Auto,
            dump_guest_core: OnOff::On,
            vga: VgaKind::Std,
            display_gl: OnOff::Off,
            nic_model: NicModel::VirtioNet,
            net_backend: NetBackend::User,
            audio_dev,
            sound_hw: SoundHw::Disabled,
            usb_device: UsbDevice::Tablet,
            boot_menu: BootMenu::Off,
            boot_strict: BootStrict::Off,
            drive_cache: DriveCache::Writeback,
            drive_aio: DriveAio::Threads,
            drive_if: DriveIf::Virtio,
            rtc_base: RtcBase::Utc,
            watchdog: WatchdogKind::Off,
            watchdog_action: WatchdogAction::Reset,
            mem_slots: 0,
            mem_maxmb: 0,
            monitor_width: 0,
            monitor_height: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct QemuImgResult {
    success: bool,
    message: String,
}

struct QcowCreateState {
    open: bool,
    path: String,
    size_gb: u32,
    backing_file: String,
    error: Option<String>,
    running: bool,
}

impl Default for QcowCreateState {
    fn default() -> Self {
        Self {
            open: false,
            path: String::new(),
            size_gb: 20,
            backing_file: String::new(),
            error: None,
            running: false,
        }
    }
}

struct QemuGui {
    config: Config,
    config_path: PathBuf,
    log: String,
    error: Option<String>,
    confirming_kill: bool,
    child: Option<Child>,
    qcow_create: QcowCreateState,
    qemu_img_result: Arc<Mutex<Option<QemuImgResult>>>,
    child_exit_code: Option<i32>,
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
            error: None,
            confirming_kill: false,
            child: None,
            qcow_create: QcowCreateState::default(),
            qemu_img_result: Arc::new(Mutex::new(None)),
            child_exit_code: None,
        }
    }
}

impl QemuGui {
    fn save_config(&mut self) {
        if let Err(e) = self.validate_basic() {
            self.log_push(&format!("Config not saved — {e}\n"));
            return;
        }
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

    fn validate_basic(&self) -> Result<(), String> {
        if self.config.qemu_path.trim().is_empty() {
            return Err("QEMU path is empty".into());
        }
        if self.config.disk_path.trim().is_empty() {
            return Err("Disk image path is empty".into());
        }
        if self.config.ram_mb == 0 {
            return Err("RAM must be > 0".into());
        }
        if self.config.smp == 0 {
            return Err("SMP must be > 0".into());
        }
        Ok(())
    }

    fn validate(&self, boot_from_cd: bool) -> Result<(), String> {
        self.validate_basic()?;
        if !std::path::Path::new(&self.config.qemu_path).exists() {
            return Err("QEMU binary not found at configured path".into());
        }
        if !std::path::Path::new(&self.config.disk_path).exists() {
            return Err("Disk image not found at configured path".into());
        }
        if boot_from_cd {
            if self.config.iso_path.trim().is_empty() {
                return Err("ISO path is empty".into());
            }
            if !std::path::Path::new(&self.config.iso_path).exists() {
                return Err("ISO file not found at configured path".into());
            }
        }
        Ok(())
    }

    fn build_args(&self, boot_from_cd: bool) -> Vec<String> {
        let mut args = Vec::new();

        // -machine
        let mut machine = vec![self.config.machine_type.as_str().to_string()];
        machine.push(format!("accel={}", self.config.accel.as_str()));
        if self.config.kernel_irqchip != IrqChip::On {
            machine.push(format!(
                "kernel-irqchip={}",
                self.config.kernel_irqchip.as_str()
            ));
        }
        if self.config.vmport != VmPort::Auto {
            machine.push(format!("vmport={}", self.config.vmport.as_str()));
        }
        if self.config.dump_guest_core != OnOff::On {
            machine.push(format!(
                "dump-guest-core={}",
                self.config.dump_guest_core.as_str()
            ));
        }
        args.push("-machine".to_string());
        args.push(machine.join(","));

        // -cpu
        if !self.config.cpu_model.is_empty() {
            args.push("-cpu".to_string());
            args.push(self.config.cpu_model.clone());
        }

        // -m
        if self.config.mem_slots > 0 && self.config.mem_maxmb > 0 {
            args.push("-m".to_string());
            args.push(format!(
                "{}M,slots={},maxmem={}M",
                self.config.ram_mb, self.config.mem_slots, self.config.mem_maxmb
            ));
        } else {
            args.push("-m".to_string());
            args.push(format!("{}M", self.config.ram_mb));
        }

        // -smp
        args.push("-smp".to_string());
        args.push(self.config.smp.to_string());

        // -drive
        args.push("-drive".to_string());
        args.push(format!(
            "file={},if={},format=qcow2,cache={},aio={}",
            self.config.disk_path,
            self.config.drive_if.as_str(),
            self.config.drive_cache.as_str(),
            self.config.drive_aio.as_str(),
        ));

        if boot_from_cd {
            args.push("-cdrom".to_string());
            args.push(self.config.iso_path.clone());
        }

        // -boot
        if !self.config.boot_order.is_empty() {
            let mut boot = format!("order={}", self.config.boot_order);
            if self.config.boot_menu == BootMenu::On {
                boot.push_str(",menu=on");
            }
            if self.config.boot_strict == BootStrict::On {
                boot.push_str(",strict=on");
            }
            args.push("-boot".to_string());
            args.push(boot);
        }

        // Network
        args.push("-netdev".to_string());
        args.push(format!("{},id=net0", self.config.net_backend.as_str()));
        args.push("-device".to_string());
        args.push(format!("{},netdev=net0", self.config.nic_model.as_str()));

        // -device <vga> (with optional xres/yres) or -vga fallback
        match self.config.vga {
            VgaKind::Vmware | VgaKind::Cirrus | VgaKind::Disabled => {
                if self.config.vga != VgaKind::Disabled {
                    args.push("-vga".to_string());
                    args.push(self.config.vga.as_str().to_string());
                }
            }
            _ => {
                let device = match self.config.vga {
                    VgaKind::Std => "VGA",
                    VgaKind::Virtio => "virtio-vga",
                    VgaKind::Qxl => "qxl",
                    _ => unreachable!(),
                };
                let mut dev = device.to_string();
                if self.config.monitor_width > 0 && self.config.monitor_height > 0 {
                    dev.push_str(&format!(
                        ",xres={},yres={}",
                        self.config.monitor_width, self.config.monitor_height
                    ));
                }
                args.push("-device".to_string());
                args.push(dev);
            }
        }

        // -display
        let mut display = self.config.display.as_str().to_string();
        if self.config.display != DisplayKind::Disabled && self.config.display_gl == OnOff::On {
            display.push_str(",gl=on");
        }
        args.push("-display".to_string());
        args.push(display);

        // USB
        if self.config.usb_device != UsbDevice::Disabled {
            args.push("-usb".to_string());
            args.push("-device".to_string());
            args.push(self.config.usb_device.as_str().to_string());
        }

        // Audio
        if self.config.audio_dev != AudioDev::Disabled {
            args.push("-audiodev".to_string());
            args.push(format!("{},id=audio0", self.config.audio_dev.as_str()));
            if self.config.sound_hw != SoundHw::Disabled {
                args.push("-device".to_string());
                args.push(format!("{},audiodev=audio0", self.config.sound_hw.as_str()));
            }
        }

        // -rtc
        args.push("-rtc".to_string());
        args.push(format!("base={}", self.config.rtc_base.as_str()));

        // Watchdog
        if !self.config.watchdog.as_str().is_empty() {
            args.push("-watchdog".to_string());
            args.push(self.config.watchdog.as_str().to_string());
            args.push("-watchdog-action".to_string());
            args.push(self.config.watchdog_action.as_str().to_string());
        }

        // Custom args
        args.extend(self.config.custom_args.split_whitespace().map(String::from));

        args
    }

    fn launch_qemu(&mut self, boot_from_cd: bool) {
        self.child_exit_code = None;
        self.error = None;
        if let Err(e) = self.validate(boot_from_cd) {
            self.error = Some(e.clone());
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
                let msg = format!("Failed to launch QEMU: {e}");
                self.log_push(&format!("{msg}\n"));
                self.error = Some(msg);
            }
        }
    }

    fn kill_qemu(&mut self) {
        self.confirming_kill = false;
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            self.log_push("QEMU terminated.\n");
        }
    }

    fn derive_qemu_img_path(&self) -> String {
        let p = std::path::Path::new(&self.config.qemu_path);
        let parent = p.parent().unwrap_or(std::path::Path::new("."));
        let img_name = if cfg!(target_os = "linux") {
            "qemu-img"
        } else {
            "qemu-img.exe"
        };
        parent.join(img_name).display().to_string()
    }

    fn create_qcow(&mut self) {
        self.qcow_create.error = None;

        let qemu_img = self.derive_qemu_img_path();
        if !std::path::Path::new(&qemu_img).exists() {
            self.qcow_create.error = Some(format!("qemu-img not found at: {qemu_img}"));
            return;
        }
        if self.qcow_create.path.trim().is_empty() {
            self.qcow_create.error = Some("Output path is empty".into());
            return;
        }
        if std::path::Path::new(self.qcow_create.path.trim()).exists() {
            self.qcow_create.error = Some("File already exists".into());
            return;
        }
        if self.qcow_create.size_gb == 0 {
            self.qcow_create.error = Some("Size must be > 0 GB".into());
            return;
        }
        if !self.qcow_create.backing_file.trim().is_empty()
            && !std::path::Path::new(self.qcow_create.backing_file.trim()).exists()
        {
            self.qcow_create.error = Some("Backing file not found".into());
            return;
        }

        let mut args = vec!["create".to_string(), "-f".to_string(), "qcow2".to_string()];
        if !self.qcow_create.backing_file.trim().is_empty() {
            args.push("-b".to_string());
            args.push(self.qcow_create.backing_file.trim().to_string());
        }
        let size_str = format!("{}G", self.qcow_create.size_gb);
        args.push(self.qcow_create.path.trim().to_string());
        args.push(size_str);

        self.qcow_create.running = true;
        let result = Arc::clone(&self.qemu_img_result);

        self.log_push(&format!(
            "Creating QCOW2: {} {} …\n",
            qemu_img,
            args.join(" ")
        ));

        std::thread::spawn(move || {
            let output = Command::new(&qemu_img).args(&args).output();
            let res = match output {
                Ok(o) => {
                    if o.status.success() {
                        let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        QemuImgResult {
                            success: true,
                            message: out,
                        }
                    } else {
                        let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
                        QemuImgResult {
                            success: false,
                            message: err,
                        }
                    }
                }
                Err(e) => QemuImgResult {
                    success: false,
                    message: format!("Failed to spawn qemu-img: {e}"),
                },
            };
            *result.lock().unwrap() = Some(res);
        });
    }

    fn poll_qemu_img_result(&mut self) {
        let taken = self.qemu_img_result.lock().unwrap().take();
        let Some(result) = taken else { return };
        self.qcow_create.running = false;
        if result.success {
            let path = self.qcow_create.path.clone();
            self.qcow_create = QcowCreateState::default();
            self.config.disk_path = path.clone();
            self.log_push(&format!(
                "Created QCOW2 disk: {} ({})\n",
                path,
                result.message.trim()
            ));
        } else {
            self.qcow_create.error = Some(result.message);
        }
    }

    fn poll_child_exit(&mut self) {
        let Some(ref mut child) = self.child else {
            return;
        };
        match child.try_wait() {
            Ok(Some(status)) => {
                let code = status.code();
                self.child_exit_code = code;
                self.child = None;
                match code {
                    Some(c) => self.log_push(&format!("QEMU exited with code {c}\n")),
                    None => self.log_push("QEMU terminated by signal\n"),
                }
            }
            Ok(None) => {}
            Err(e) => {
                self.log_push(&format!("Error polling QEMU: {e}\n"));
                self.child = None;
            }
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
        self.poll_qemu_img_result();
        self.poll_child_exit();

        let log_width = ui.available_width() * 0.8;
        let right_w = 40.0;
        let min_central = 310.0;
        let max_left = (ui.available_width() - right_w - min_central).max(150.0);
        egui::Panel::left("log_panel")
            .resizable(true)
            .default_size(log_width)
            .min_size(150.0)
            .max_size(max_left)
            .show_inside(ui, |ui| {
                ui.heading("Log");
                ui.separator();
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&self.log).text_style(egui::TextStyle::Monospace),
                        );
                    });
            });

        egui::Panel::right("command_panel")
            .resizable(false)
            .default_size(40.0)
            .min_size(36.0)
            .show_inside(ui, |ui| {
                let btn = egui::Vec2::splat(28.0);
                let running = self.child.is_some();
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), ui.available_height()),
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        ui.add_space(8.0);
                        if ui
                            .add_enabled(!running, egui::Button::new("\u{25B6}").min_size(btn))
                            .on_hover_text("Boot from Disk")
                            .clicked()
                        {
                            self.confirming_kill = false;
                            self.launch_qemu(false);
                        }
                        ui.add_space(4.0);
                        if ui
                            .add_enabled(!running, egui::Button::new("\u{1F4BF}").min_size(btn))
                            .on_hover_text("Install (Boot from ISO)")
                            .clicked()
                        {
                            self.confirming_kill = false;
                            self.launch_qemu(true);
                        }
                        ui.add_space(4.0);
                        if self.confirming_kill {
                            if ui
                                .add_enabled(running, egui::Button::new("\u{26A0}").min_size(btn))
                                .on_hover_text("Confirm kill QEMU?")
                                .clicked()
                            {
                                self.kill_qemu();
                            }
                        } else if ui
                            .add_enabled(running, egui::Button::new("\u{23F9}").min_size(btn))
                            .on_hover_text("Kill QEMU")
                            .clicked()
                        {
                            self.confirming_kill = true;
                        }
                    },
                );
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("QEMU GUI Wrapper");

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("QEMU Path:");
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Executable", &["exe"])
                            .set_title("Select QEMU Executable")
                            .pick_file()
                        {
                            self.config.qemu_path = path.display().to_string();
                            self.error = None;
                        }
                    }
                });
                if ui
                    .text_edit_singleline(&mut self.config.qemu_path)
                    .changed()
                {
                    self.error = None;
                }

                ui.horizontal(|ui| {
                    ui.label("Disk Image:");
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Disk Image", &["qcow2", "img", "raw"])
                            .set_title("Select Disk Image")
                            .pick_file()
                        {
                            self.config.disk_path = path.display().to_string();
                            self.error = None;
                        }
                    }
                    if ui.button("Create QCOW2...").clicked() {
                        self.qcow_create.open = true;
                    }
                });
                if ui
                    .text_edit_singleline(&mut self.config.disk_path)
                    .changed()
                {
                    self.error = None;
                }

                ui.horizontal(|ui| {
                    ui.label("ISO Path:");
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("ISO", &["iso"])
                            .set_title("Select ISO File")
                            .pick_file()
                        {
                            self.config.iso_path = path.display().to_string();
                            self.error = None;
                        }
                    }
                });
                if ui.text_edit_singleline(&mut self.config.iso_path).changed() {
                    self.error = None;
                }

                ui.horizontal(|ui| {
                    ui.label("RAM (MB):");
                    ui.add(egui::Slider::new(&mut self.config.ram_mb, 512..=32768))
                        .on_hover_text("Memory allocated to the VM (512 MB – 32 GB)");
                });

                ui.horizontal(|ui| {
                    ui.label("SMP (CPU cores):");
                    ui.add(egui::Slider::new(&mut self.config.smp, 1..=32))
                        .on_hover_text("Number of CPU cores assigned to the VM");
                });

                ui.horizontal(|ui| {
                    ui.label("Display:");
                    egui::ComboBox::from_id_salt("display")
                        .selected_text(self.config.display.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.config.display, DisplayKind::Gtk, "GTK");
                            ui.selectable_value(&mut self.config.display, DisplayKind::Sdl, "SDL");
                            ui.selectable_value(&mut self.config.display, DisplayKind::Disabled, "None");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Boot order:");
                    let boot_orders = ["c", "d", "cd", "dc", "ncd", "cnd"];
                    egui::ComboBox::from_id_salt("boot_order")
                        .selected_text(&self.config.boot_order)
                        .show_ui(ui, |ui| {
                            for &o in &boot_orders {
                                ui.selectable_value(&mut self.config.boot_order, o.to_string(), o);
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(format!("Config: {}", self.config_path.display()));
                    if ui.button("Save Config").clicked() {
                        self.save_config();
                    }
                });

                if let Some(ref err) = self.error {
                    ui.colored_label(egui::Color32::RED, err.as_str());
                }

                egui::CollapsingHeader::new("Machine")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt("machine_type")
                                .selected_text(self.config.machine_type.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.machine_type, MachineType::Pc, "pc (i440FX)");
                                    ui.selectable_value(&mut self.config.machine_type, MachineType::Q35, "q35 (ICH9)");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("CPU model:");
                            ui.text_edit_singleline(&mut self.config.cpu_model)
                                .on_hover_text("Common: host, max, qemu64, SandyBridge, Haswell, Skylake, EPYC");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Accelerator:");
                            egui::ComboBox::from_id_salt("accel")
                                .selected_text(self.config.accel.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.accel, AccelKind::Kvm, "kvm");
                                    ui.selectable_value(&mut self.config.accel, AccelKind::Whpx, "whpx");
                                    ui.selectable_value(&mut self.config.accel, AccelKind::Tcg, "tcg");
                                    ui.selectable_value(&mut self.config.accel, AccelKind::Hax, "hax");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("kernel-irqchip:");
                            egui::ComboBox::from_id_salt("irqchip")
                                .selected_text(self.config.kernel_irqchip.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.kernel_irqchip, IrqChip::Off, "off");
                                    ui.selectable_value(&mut self.config.kernel_irqchip, IrqChip::Split, "split");
                                    ui.selectable_value(&mut self.config.kernel_irqchip, IrqChip::On, "on");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("vmport:");
                            egui::ComboBox::from_id_salt("vmport")
                                .selected_text(self.config.vmport.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.vmport, VmPort::Auto, "auto");
                                    ui.selectable_value(&mut self.config.vmport, VmPort::On, "on");
                                    ui.selectable_value(&mut self.config.vmport, VmPort::Off, "off");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("dump-guest-core:");
                            egui::ComboBox::from_id_salt("dump_guest_core")
                                .selected_text(self.config.dump_guest_core.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.dump_guest_core, OnOff::On, "on");
                                    ui.selectable_value(&mut self.config.dump_guest_core, OnOff::Off, "off");
                                });
                        });
                    });

                egui::CollapsingHeader::new("Storage")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Drive cache:");
                            egui::ComboBox::from_id_salt("drive_cache")
                                .selected_text(self.config.drive_cache.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.drive_cache, DriveCache::Writeback, "writeback");
                                    ui.selectable_value(&mut self.config.drive_cache, DriveCache::Disabled, "none");
                                    ui.selectable_value(&mut self.config.drive_cache, DriveCache::Writethrough, "writethrough");
                                    ui.selectable_value(&mut self.config.drive_cache, DriveCache::Unsafe, "unsafe");
                                    ui.selectable_value(&mut self.config.drive_cache, DriveCache::Directsync, "directsync");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Drive AIO:");
                            egui::ComboBox::from_id_salt("drive_aio")
                                .selected_text(self.config.drive_aio.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.drive_aio, DriveAio::Threads, "threads");
                                    ui.selectable_value(&mut self.config.drive_aio, DriveAio::Native, "native");
                                    ui.selectable_value(&mut self.config.drive_aio, DriveAio::IoUring, "io_uring");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Drive interface:");
                            egui::ComboBox::from_id_salt("drive_if")
                                .selected_text(self.config.drive_if.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.drive_if, DriveIf::Virtio, "virtio");
                                    ui.selectable_value(&mut self.config.drive_if, DriveIf::Ide, "ide");
                                    ui.selectable_value(&mut self.config.drive_if, DriveIf::Sata, "sata");
                                    ui.selectable_value(&mut self.config.drive_if, DriveIf::Sd, "sd");
                                });
                        });
                    });

                egui::CollapsingHeader::new("Network")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("NIC model:");
                            egui::ComboBox::from_id_salt("nic_model")
                                .selected_text(self.config.nic_model.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.nic_model, NicModel::VirtioNet, "virtio-net");
                                    ui.selectable_value(&mut self.config.nic_model, NicModel::E1000, "e1000");
                                    ui.selectable_value(&mut self.config.nic_model, NicModel::Rtl8139, "rtl8139");
                                    ui.selectable_value(&mut self.config.nic_model, NicModel::Ne2kPci, "ne2k_pci");
                                    ui.selectable_value(&mut self.config.nic_model, NicModel::Pcnet, "pcnet");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Backend:");
                            egui::ComboBox::from_id_salt("net_backend")
                                .selected_text(self.config.net_backend.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.net_backend, NetBackend::User, "user (SLIRP)");
                                    ui.selectable_value(&mut self.config.net_backend, NetBackend::Tap, "tap");
                                });
                        });
                    });

                egui::CollapsingHeader::new("Display Options")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("VGA:");
                            egui::ComboBox::from_id_salt("vga")
                                .selected_text(self.config.vga.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.vga, VgaKind::Std, "std");
                                    ui.selectable_value(&mut self.config.vga, VgaKind::Virtio, "virtio");
                                    ui.selectable_value(&mut self.config.vga, VgaKind::Qxl, "qxl");
                                    ui.selectable_value(&mut self.config.vga, VgaKind::Vmware, "vmware");
                                    ui.selectable_value(&mut self.config.vga, VgaKind::Cirrus, "cirrus");
                                    ui.selectable_value(&mut self.config.vga, VgaKind::Disabled, "none");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("GL acceleration:");
                            egui::ComboBox::from_id_salt("display_gl")
                                .selected_text(self.config.display_gl.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.display_gl, OnOff::Off, "off");
                                    ui.selectable_value(&mut self.config.display_gl, OnOff::On, "on");
                                });
                        });
                        // -- Monitor resolution --
                        const PRESETS: &[(&str, u32, u32)] = &[
                            ("Auto (default)", 0, 0),
                            ("640×480", 640, 480),
                            ("800×600", 800, 600),
                            ("1024×768", 1024, 768),
                            ("1280×720", 1280, 720),
                            ("1366×768", 1366, 768),
                            ("1920×1080", 1920, 1080),
                            ("2560×1440", 2560, 1440),
                            ("3840×2160", 3840, 2160),
                        ];
                        let custom_idx = PRESETS.len();
                        let cur = (self.config.monitor_width, self.config.monitor_height);
                        let preset_idx = PRESETS
                            .iter()
                            .position(|&(_, w, h)| (w, h) == cur)
                            .unwrap_or(custom_idx);
                        let preset_label = if preset_idx < custom_idx {
                            PRESETS[preset_idx].0
                        } else {
                            "Custom"
                        };
                        ui.horizontal(|ui| {
                            ui.label("Resolution:");
                            egui::ComboBox::from_id_salt("resolution_preset")
                                .selected_text(preset_label)
                                .show_ui(ui, |ui| {
                                    for (i, &(name, w, h)) in PRESETS.iter().enumerate() {
                                        if ui.selectable_label(preset_idx == i, name).clicked() {
                                            self.config.monitor_width = w;
                                            self.config.monitor_height = h;
                                        }
                                    }
                                    let is_custom = preset_idx == custom_idx;
                                    if ui.selectable_label(is_custom, "Custom").clicked() && cur == (0, 0) {
                                        self.config.monitor_width = 1024;
                                        self.config.monitor_height = 768;
                                    }
                                });
                        });
                        if preset_idx == custom_idx {
                            ui.horizontal(|ui| {
                                ui.label("Width:");
                                ui.add(egui::DragValue::new(&mut self.config.monitor_width).range(256..=7680));
                                ui.label("Height:");
                                ui.add(egui::DragValue::new(&mut self.config.monitor_height).range(256..=7680));
                            });
                        }
                    });

                egui::CollapsingHeader::new("Audio")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Backend:");
                            egui::ComboBox::from_id_salt("audio_dev")
                                .selected_text(self.config.audio_dev.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.audio_dev, AudioDev::Disabled, "none");
                                    ui.selectable_value(&mut self.config.audio_dev, AudioDev::Pa, "pa");
                                    ui.selectable_value(&mut self.config.audio_dev, AudioDev::Alsa, "alsa");
                                    ui.selectable_value(&mut self.config.audio_dev, AudioDev::Dsound, "dsound");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Device:");
                            egui::ComboBox::from_id_salt("sound_hw")
                                .selected_text(self.config.sound_hw.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.sound_hw, SoundHw::Disabled, "none");
                                    ui.selectable_value(&mut self.config.sound_hw, SoundHw::IntelHda, "intel-hda");
                                    ui.selectable_value(&mut self.config.sound_hw, SoundHw::Ac97, "ac97");
                                    ui.selectable_value(&mut self.config.sound_hw, SoundHw::Sb16, "sb16");
                                    ui.selectable_value(&mut self.config.sound_hw, SoundHw::Es1370, "es1370");
                                });
                        });
                    });

                egui::CollapsingHeader::new("Advanced")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("USB device:");
                            egui::ComboBox::from_id_salt("usb_device")
                                .selected_text(self.config.usb_device.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.usb_device, UsbDevice::Tablet, "usb-tablet");
                                    ui.selectable_value(&mut self.config.usb_device, UsbDevice::Mouse, "usb-mouse");
                                    ui.selectable_value(&mut self.config.usb_device, UsbDevice::Disabled, "none");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Boot menu:");
                            egui::ComboBox::from_id_salt("boot_menu")
                                .selected_text(self.config.boot_menu.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.boot_menu, BootMenu::Off, "off");
                                    ui.selectable_value(&mut self.config.boot_menu, BootMenu::On, "on");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Boot strict:");
                            egui::ComboBox::from_id_salt("boot_strict")
                                .selected_text(self.config.boot_strict.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.boot_strict, BootStrict::Off, "off");
                                    ui.selectable_value(&mut self.config.boot_strict, BootStrict::On, "on");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("RTC base:");
                            egui::ComboBox::from_id_salt("rtc_base")
                                .selected_text(self.config.rtc_base.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.rtc_base, RtcBase::Utc, "utc");
                                    ui.selectable_value(&mut self.config.rtc_base, RtcBase::Localtime, "localtime");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Watchdog:");
                            egui::ComboBox::from_id_salt("watchdog")
                                .selected_text(if self.config.watchdog.as_str().is_empty() { "off" } else { self.config.watchdog.as_str() })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.watchdog, WatchdogKind::Off, "off");
                                    ui.selectable_value(&mut self.config.watchdog, WatchdogKind::I6300esb, "i6300esb");
                                    ui.selectable_value(&mut self.config.watchdog, WatchdogKind::Ib700, "ib700");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Watchdog action:");
                            egui::ComboBox::from_id_salt("watchdog_action")
                                .selected_text(self.config.watchdog_action.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.config.watchdog_action, WatchdogAction::Reset, "reset");
                                    ui.selectable_value(&mut self.config.watchdog_action, WatchdogAction::Shutdown, "shutdown");
                                    ui.selectable_value(&mut self.config.watchdog_action, WatchdogAction::Poweroff, "poweroff");
                                    ui.selectable_value(&mut self.config.watchdog_action, WatchdogAction::InjectNmi, "inject-nmi");
                                    ui.selectable_value(&mut self.config.watchdog_action, WatchdogAction::Disabled, "none");
                                    ui.selectable_value(&mut self.config.watchdog_action, WatchdogAction::Pause, "pause");
                                    ui.selectable_value(&mut self.config.watchdog_action, WatchdogAction::Debug, "debug");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Memory hotplug slots:");
                            ui.add(egui::Slider::new(&mut self.config.mem_slots, 0..=16))
                                .on_hover_text("Number of hotpluggable memory slots (0 = disabled)");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Max memory (MB):");
                            ui.add(egui::Slider::new(&mut self.config.mem_maxmb, 0..=262144))
                                .on_hover_text("Maximum addressable memory (0 = same as RAM)");
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Extra QEMU args:");
                            if ui.button("Clear").clicked() {
                                self.config.custom_args.clear();
                            }
                        });
                        ui.text_edit_multiline(&mut self.config.custom_args)
                            .on_hover_text("Additional arguments passed verbatim to QEMU,\ne.g. -msg timestamp=on -no-hpet");
                    });

                if self.qcow_create.open {
                    egui::Window::new("Create QCOW2 Disk Image")
                        .collapsible(false)
                        .resizable(false)
                        .show(ui.ctx(), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Path:");
                                if ui.button("Browse...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("QCOW2", &["qcow2"])
                                        .set_title("Save QCOW2 Disk Image")
                                        .save_file()
                                    {
                                        self.qcow_create.path = path.display().to_string();
                                    }
                                }
                            });
                            ui.text_edit_singleline(&mut self.qcow_create.path);

                            ui.horizontal(|ui| {
                                ui.label("Size (GB):");
                                ui.add(egui::Slider::new(&mut self.qcow_create.size_gb, 1..=500));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Backing file (optional):");
                                if ui.button("Browse...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Disk Image", &["qcow2", "img", "raw"])
                                        .set_title("Select Backing File")
                                        .pick_file()
                                    {
                                        self.qcow_create.backing_file = path.display().to_string();
                                    }
                                }
                            });
                            ui.text_edit_singleline(&mut self.qcow_create.backing_file);
                            ui.label(
                                egui::RichText::new("Base image for copy-on-write; leave empty for a standalone disk")
                                    .color(egui::Color32::GRAY)
                                    .size(11.0),
                            );

                            ui.separator();

                            if let Some(ref err) = self.qcow_create.error {
                                ui.colored_label(egui::Color32::RED, err);
                            }

                            ui.horizontal(|ui| {
                                let creating = self.qcow_create.running;
                                if ui
                                    .add_enabled(!creating, egui::Button::new("Create"))
                                    .clicked()
                                {
                                    self.create_qcow();
                                }
                                if ui.button("Cancel").clicked() {
                                    self.qcow_create = QcowCreateState::default();
                                }
                            });
                        });
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([880.0, 600.0])
            .with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "QEMU GUI",
        options,
        Box::new(|_cc| Ok(Box::new(QemuGui::default()))),
    )
}
