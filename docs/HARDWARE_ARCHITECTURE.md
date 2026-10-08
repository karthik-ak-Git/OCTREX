# OCTREX Phase 5 — Hardware Intelligence & Model Compatibility Architecture

## Overview

The Hardware Intelligence layer (`crates/octrex-core/src/hardware/`) is a core subsystem of Octrex. It provides real-time, deterministic, and safe local hardware detection and model compatibility evaluation without external network calls or risky shell executions.

> **CRITICAL ARCHITECTURAL BOUNDARY:**  
> **Hardware compatibility is NOT model routing.**  
> The hardware layer determines *what a machine can physically and reasonably run*. It does **NOT** select models, enforce privacy policies, block network traffic, or perform cloud fallbacks. Final routing decisions belong to the future Model Router.

---

## Architecture Overview

```
                                USER REQUEST
                                     │
                                     ▼
                             ┌──────────────┐
                             │ Future Router│
                             └──────┬───────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
       Model Descriptor (Phase 4)       Hardware Profile (Phase 5)
                    │                               │
                    └───────────────┬───────────────┘
                                    ▼
                       Model Compatibility Engine
                                    │
                                    ▼
                           Compatibility Result
                     (Compatible / Warning / Incompatible)
```

---

## Key Components

### 1. `HardwareProfile` & `HardwareSnapshot` (`profile.rs`)
- `HardwareProfile`: Represents relatively stable machine characteristics including Operating System, Architecture, CPU details, Memory capacity, and GPU information.
- `HardwareSnapshot`: Captures dynamic runtime metric state (RAM usage %, CPU load %, VRAM utilization).
- Units: System memory and VRAM are stored internally as exact byte integers (`u64`), avoiding arbitrary string formatting.

### 2. Hardware Detection Engine (`cpu.rs`, `memory.rs`, `gpu.rs`, `platform.rs`)
- **CPU**: Detects logical core count, physical cores, vendor name, model brand, and instruction set extensions (e.g. AVX2, AVX512, FMA).
- **RAM**: Detects total RAM, available RAM, used RAM, and Apple Silicon Unified Memory flag (`is_unified_memory`).
- **GPU Subsystem**:
  - Distinguishes NVIDIA, AMD, Intel, Apple, and Unknown accelerators.
  - Subprocess execution (e.g., `nvidia-smi`) uses direct `tokio::process::Command` with fixed executable arguments (NO shell parsing, strict 2-second timeout, defensive CSV parsing).
  - Graceful degradation: A machine without a GPU produces a valid `HardwareProfile` with 0 GPUs. Missing drivers or VM environments degrade to `Unknown` confidence without throwing exceptions.

### 3. Hardware Service (`detector.rs`)
- Extends `ApplicationState` and registers under `ServiceRegistry` as `"hardware_service"`.
- Uses `HardwareProbe` trait (`RealHardwareProbe` and `MockHardwareProbe`), making hardware testable deterministically.
- Publishes lifecycle events on the `EventBus`:
  - `HARDWARE_DETECTION_STARTED`
  - `HARDWARE_DETECTION_COMPLETED`
  - `HARDWARE_DETECTION_FAILED`
  - `HARDWARE_PROFILE_CHANGED`
  - `MODEL_COMPATIBILITY_CHECKED`

### 4. Model Compatibility Engine (`compatibility.rs`)
Consumes Phase 4 `ModelDescriptor` and evaluates compatibility against `HardwareProfile`:

- **States**:
  - `Compatible`: All hardware requirements met.
  - `CompatibleWithWarnings`: Model can run (e.g., on CPU or with reduced VRAM offload), but performance may be constrained.
  - `Incompatible`: Hard incompatibility (e.g. insufficient RAM/VRAM, required GPU missing, vendor mismatch, OS mismatch).
  - `Unknown`: Requirements are incomplete or unspecified.

- **Deterministic Evaluation Rules**:
  1. *Platform check*: Ensures OS matches required platform metadata.
  2. *Architecture check*: Ensures CPU architecture matches requirements.
  3. *GPU Requirement*: Rejects GPU-only models if no GPU is present.
  4. *Vendor Match*: Rejects models requiring specific GPU vendor if mismatched.
  5. *RAM Check*: Compares system RAM against `minimum_ram` and `recommended_ram`.
  6. *VRAM Check*: Compares GPU VRAM against `minimum_vram`. Offloads to CPU with warning if model permits.
  7. *CPU Execution*: CPU-only machines receive explicit performance warnings rather than global rejection.

### 5. Hardware Profiler (`profiler.rs`)
Lightweight performance snapshot interface capturing real-time RAM/CPU/GPU utilization. Non-intrusive and does not launch benchmark inference runs.

---

## Backend REST Endpoints (`octrex-server`)

- `GET /api/hardware`: Returns the active `HardwareProfile` and current runtime `HardwareSnapshot`.
- `POST /api/hardware/refresh` / `GET /api/hardware/refresh`: Forces cache invalidation and hardware re-detection.
- `GET /api/hardware/compatibility?model_id=<ID>`: Evaluates compatibility for a single model or all registered models.

---

## Integration Points

- **Phase 1**: Integrates into `ApplicationState` and `ServiceRegistry`.
- **Phase 2**: Settings/preferences use SQLite; hardware profiles are runtime-derived and never saved as static DB files.
- **Phase 3**: Preserves workspace and filesystem security boundaries.
- **Phase 4**: Consumes Phase 4 `ModelDescriptor`, `ModelRequirement`, `HardwareRequirement`, and `ExecutionMode`.
- **Future Phase 6 (Router)**: Feeds `CompatibilityResult` into the final routing matrix alongside privacy policy and user preferences.
