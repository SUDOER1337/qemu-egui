
# QEMU Frontend Investigation Report

## Summary

A NixOS virtual machine created on Linux was migrated to Windows and launched using a custom Rust/EGUI QEMU frontend. The VM failed to boot. Initial investigation focused on WHPX, APX/MPX CPU warnings, and graphics configuration. Final analysis suggests the primary issue is likely a UEFI/BIOS mismatch and/or an incorrect or incomplete QCOW2 image rather than CPU feature incompatibilities.

---

# Environment

## Host System

CPU:
Intel Core i5-661 @ 3.33 GHz

Hypervisor:
WHPX (Windows Hypervisor Platform)

QEMU Version:
11.0.50 (v11.0.0-12631-g54e84cdc7a)

Guest OS:
NixOS

Disk Format:
QCOW2

Disk Path:
D:\qemu\vms\nixos\nix.qcow2

---

# Original Command

```bash
D:\qemu\qemu-system-x86_64.exe \
-machine pc,accel=whpx,vmport=on \
-cpu host \
-m 4096M \
-smp 4 \
-drive file=D:\qemu\vms\nixos\nix.qcow2,if=virtio,format=qcow2,cache=writeback,aio=threads \
-boot order=c \
-netdev user,id=net0 \
-device virtio-net,netdev=net0 \
-device virtio-vga \
-display gtk,gl=on \
-usb \
-device usb-tablet \
-rtc base=utc
```

---

# Observed Warnings

## APX / MPX Conflicts

Observed:

```text
feature conflicts with APX
feature conflicts with MPX
```

Analysis:

* Host CPU predates APX by more than a decade.
* Warnings originate from QEMU CPU model generation.
* Warnings are likely cosmetic.
* No evidence that these warnings caused boot failure.

Conclusion:

Low diagnostic value.

---

## WHPX XSAVE Warning

Observed:

```text
WHPX: Partition is not XSAVE capable
```

Analysis:

Common WHPX warning.

Conclusion:

Likely harmless.

---

## WHPX Performance Monitoring Warning

Observed:

```text
Failed to get performance monitoring features
```

Analysis:

Common WHPX limitation.

Conclusion:

Likely harmless.

---

## WHPX VP Exit Code 4

Observed only when using:

```bash
-cpu host
```

Disappeared when using:

```bash
-cpu qemu64
```

Analysis:

Suggests incompatibility between host CPU passthrough and WHPX.

Conclusion:

Possible WHPX CPU-model issue.

Recommended frontend behavior:

Prefer:

```bash
-cpu qemu64
```

or

```bash
-cpu max
```

over:

```bash
-cpu host
```

when WHPX is selected.

---

# Graphics Investigation

Tested:

```bash
-device virtio-vga
```

and

```bash
-vga std
```

No meaningful change.

Conclusion:

Graphics configuration not responsible for boot failure.

---

# Firmware Investigation

Original launch used no explicit firmware.

Behavior:

```text
No bootable devices
```

Observation:

Linux installation was likely performed in UEFI mode.

Windows launch defaulted to SeaBIOS.

This creates a likely BIOS/UEFI mismatch.

---

# EDK2 Discovery

Located:

```text
D:\qemu\share\edk2-x86_64-code.fd
```

Attempted:

```bash
-bios edk2-x86_64-code.fd
```

Result:

```text
could not load PC BIOS
```

Correct method:

```bash
-drive if=pflash,format=raw,readonly=on,file=edk2-x86_64-code.fd
```

Result:

UEFI screen successfully appeared.

This confirms:

* EDK2 firmware is functional.
* Guest can enter UEFI environment.

---

# Disk Image Analysis

Command:

```bash
qemu-img info nix.qcow2
```

Result:

```text
virtual size: 30 GiB
disk size: 192 KiB
```

Analysis:

192 KiB is extremely small for an installed NixOS system.

Possible explanations:

1. Wrong QCOW2 selected.
2. Missing backing file.
3. Installation never written.
4. Overlay image copied without base image.

Confidence:

High.

This became the strongest indicator that the disk contents may not match expectations.

---

# Diagnostic Findings

## High Confidence

* BIOS/UEFI mismatch existed.
* EDK2 firmware available and functional.
* WHPX works sufficiently to launch guests.
* Host CPU passthrough causes additional instability.
* QCOW2 size appears suspiciously small.

## Medium Confidence

* Guest installation may be missing.
* Guest installation may exist in a backing file.
* Wrong image may have been selected.

## Low Confidence

* APX/MPX warnings responsible for failure.
* VirtIO GPU responsible for failure.
* GTK frontend responsible for failure.

---

# Recommended Frontend Features

## Firmware Mode Selection

Expose explicit firmware options:

* SeaBIOS
* EDK2 / UEFI

Generated command examples:

BIOS:

```bash
-machine pc
```

UEFI:

```bash
-drive if=pflash,format=raw,readonly=on,file=edk2-x86_64-code.fd
```

---

## Automatic Disk Validation

When a disk is selected:

Run:

```bash
qemu-img info
```

Display:

* Virtual size
* Actual size
* Backing file
* Snapshot information

Warn if:

```text
disk size << virtual size
```

Example:

```text
Warning:
QCOW2 image is only 192 KiB.
This may indicate:
- Empty installation
- Missing backing file
- Incorrect image selected
```

---

## CPU Compatibility Presets

Expose presets:

Maximum Compatibility

```bash
-cpu qemu64
```

Modern Generic

```bash
-cpu max
```

Host Passthrough

```bash
-cpu host
```

Recommend compatibility mode by default on WHPX.

---

## Hypervisor Diagnostics

Parse stderr and classify warnings.

Example:

| Pattern                        | Severity |
| ------------------------------ | -------- |
| APX/MPX conflict               | Low      |
| XSAVE warning                  | Low      |
| Performance monitoring warning | Low      |
| VP exit code 4                 | Medium   |
| No bootable devices            | High     |

---

## Boot Failure Assistant

If firmware reports:

```text
No bootable devices
```

Suggest:

1. Verify firmware type.
2. Check BIOS vs UEFI mismatch.
3. Check QCOW2 backing files.
4. Open UEFI shell.
5. Inspect EFI partitions.

---

# Final Assessment

The most likely causes of failure were:

1. BIOS/UEFI mismatch.
2. Incorrect, incomplete, or overlay QCOW2 image.
3. Host CPU passthrough instability under WHPX.

The APX/MPX warnings initially appeared important but were ultimately identified as low-value diagnostic noise.

The most actionable indicators during debugging were:

* "No bootable devices"
* Successful EDK2 startup
* QCOW2 image size of 192 KiB

Future frontend development should prioritize firmware awareness, disk validation, and warning classification over raw stderr display.
