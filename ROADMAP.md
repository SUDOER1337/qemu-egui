# Roadmap — Multiple Virtual Drives

## Goal

Replace the single `disk_path` with a list of drives so the VM can attach multiple disk
images on different buses (virtio, ide, sata, sd).

---

## 1. New data types

### `DriveConfig` (serde)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DriveConfig {
    path: String,
    iface: DriveIf,
}
impl Default for DriveConfig {
    fn default() -> Self {
        Self { path: String::new(), iface: DriveIf::Virtio }
    }
}
```

### Config changes

| Remove                    | Add                         |
|---------------------------|-----------------------------|
| `disk_path: String`       | `disks: Vec<DriveConfig>`   |
| `drive_if: DriveIf`       | *(moved into DriveConfig)*  |

**Default** — one empty entry: `vec![DriveConfig::default()]`.

### Backward-compat
Old config files have `disk_path` / `drive_if` fields that no longer exist in the
struct. Serde silently ignores unknown fields, and `disks` defaults to `vec![DriveConfig::default()]`.
The user picks their drive path once after upgrading. No migration code.

---

## 2. Validation (`fn validate` / `fn validate_basic`)

- `validate_basic()`: at least one `drive.path` in `disks` must be non-empty.
- `validate()`: every non-empty `drive.path` must point to an existing file.

Disk boot (the ▶ button) requires the first non-empty drive; ISO boot skips disk
validation entirely (already the case).

---

## 3. `build_args()` — drive emission

```rust
for (i, drive) in self.config.disks.iter().enumerate() {
    if drive.path.trim().is_empty() {
        continue;
    }
    args.push("-drive".to_string());
    args.push(format!(
        "file={},if={},index={i},cache={},aio={},format=qcow2",
        drive.path,
        drive.iface.as_str(),
        self.config.drive_cache.as_str(),
        self.config.drive_aio.as_str(),
    ));
}
```

- `cache` and `aio` stay global (shared defaults).
- `index=N` ensures QEMU distinguishes drives on the same bus. Two drives with
  `if=virtio,index=0` and `if=virtio,index=1` get separate virtio-blk devices.
- First non-empty drive = primary boot device (boot order `c` points to it).

---

## 4. Disk info per drive

Replace the single `disk_info_result: Arc<Mutex<Option<DiskInfoResult>>>` with:

```rust
disk_info_cache: Arc<Mutex<HashMap<String, DiskInfoResult>>>,
```

When a drive path changes (or on first render):
1. Check if an entry already exists in the cache for that path.
2. If missing/stale, spawn a thread running `qemu-img info "<path>"`.
3. Thread writes `DiskInfoResult` into the hashmap keyed by path.

On each frame, `poll_disk_info_cache()` merges completed results.
UI looks up by path — if absent, shows a gray "Checking disk…" label.

---

## 5. UI — Storage section

Replace the current "Disk Image" + "ISO Path" block inside the `► Storage` collapsing
header.

### Layout

```
► Storage
  [Drive cache: writeback▼] [Drive AIO: threads▼]

  ── Virtual Drives ────────────────────────────────
  #1  [path/to/disk1.qcow2________________] [Browse]
      Interface: [virtio▼]  [▲] [▼] [✕]
      Disk Info: qcow2, 30 GiB virtual, 4.2 GiB actual

  #2  [path/to/disk2.qcow2________________] [Browse]
      Interface: [ide▼]  [▲] [▼] [✕]
      Disk Info: qcow2, 50 GiB virtual, 12 GiB actual

  [+ Add Virtual Drive]  [Create QCOW2…]
```

### Row rules

- **Up/down arrows (▲ / ▼):** move the drive in the list. First entry has ▲ disabled.
  Last entry has ▼ disabled. No drag support — kept simple.
- **Remove (✕):** removes the drive. Disabled if `disks.len() == 1` (always keep at
  least one slot).
- **Browse:** opens file picker filtered to `["qcow2", "img", "raw"]`, sets `path`.
- **Interface combo:** per-drive `DriveIf` selection (virtio / ide / sata / sd).
- **Disk Info:** inline label, left-aligned below the text field. Shows the cached
  `DiskInfoResult` for this path. Yellow warning for sparse/small disk shown inline.

### Global actions below the list

- **`[+ Add Virtual Drive]`** — appends a new `DriveConfig::default()` to `disks`.
- **`[Create QCOW2…]`** — opens the existing create-QCOW2 window. On success, the new
  image path is appended as a new drive entry.

---

## 6. QCOW2 creation — success handler

In `poll_qemu_img_result()` (line ~913), currently:

```rust
self.config.disk_path = path.clone();  // replace
```

Change to:

```rust
self.config.disks.push(DriveConfig { path: path.clone(), ..default() });
// log "Created QCOW2 disk: …\n"
```

---

## 7. Cleanup of removed fields

- Remove `disk_path` from `Config`, `Config::default()`.
- Remove `drive_if` from `Config`, `Config::default()`.
- Remove single `disk_info_result` / `disk_info_running` from `QemuGui`.
- Remove single `run_qemu_img_info()` — replaced by per-path spawns.

---

## 8. Order of implementation

1. Define `DriveConfig`, add `disks` to `Config`, drop `disk_path` / `drive_if`.
2. Replace single disk info with `disk_info_cache` (HashMap).
3. Rewrite `build_args()` loop.
4. Rewrite validation.
5. Rewrite Storage UI section.
6. Update QCOW2 success handler.
7. Test: `just check` / `cargo build`.
