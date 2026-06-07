# MemoryKnight (`mknight`)

> A tiny, user-space "sandbox watcher" that stops a runaway `malloc` loop from
> freezing your machine — and tells you *why* it happened.

`mknight` runs your program as a child process, caps how much memory it can use,
and kills it the instant it goes out of control. Instead of a frozen desktop and
a hard reboot, you get a clean, educational post-mortem.

Built for **C/C++ students** learning dynamic memory management, where a single
misplaced `while` + `malloc()` is a rite of passage.

```text
  __  __ _          _       _     _     _____                       _
 |  \/  | |        (_)     | |   | |   |  __ \                     | |
 | \  / | | ___ __  _  __ _| |__ | |_  | |__) |___ _ __   ___  _ __| |_
 | |\/| | |/ / '_ \| |/ _` | '_ \| __| |  _  // _ \ '_ \ / _ \| '__| __|
 | |  | |   <| | | | | (_| | | | | |_  | | \ \  __/ |_) | (_) | |  | |_
 |_|  |_|_|\_\_| |_|_|\__, |_| |_|\__| |_|  \_\___| .__/ \___/|_|   \__|
                       __/ |                      | |
                      |___/                       |_|

[!] PROCESS TERMINATED TO SAVE YOUR SYSTEM

The program `./exercise` was aborted because it violated
the safety policy: [Explosive Growth Velocity Detected].

[*] Post-Mortem Analytics
    - Total Execution Time : 0.39 seconds
    - Peak RAM Consumed    : 712.00 MB (limit 1.00 GB)
    - Allocation Velocity  : ~1.95 GB/sec (Severe Leak)

[i] System Guard Insight
    Your program allocated memory explosively - a near-vertical curve.
    This almost always means a loop calling malloc() or calloc() with
    no matching free() and no exit condition. Check the bounds of your
    while/for loops and make every allocation be released.
======================================================================
```

---

## The problem

When a program leaks memory explosively (an unbounded `malloc`/`calloc` loop),
the OS reacts too late:

- The Linux kernel OOM killer only fires when RAM is nearly exhausted.
- By then the desktop environment is *thrashing* — cursor, keyboard and UI freeze.
- Your only way out is a hard reboot, losing unsaved work.

## The solution

`mknight` doesn't police the whole OS like `earlyoom` or `systemd-oomd`. It is
**developer-centric**: it supervises only the one program you ask it to run.

- 🧪 **Sandboxed** — monitors a single child process, not the system.
- 🔓 **No root** — it only constrains and kills its own children; runs in user space.
- 🎓 **Educational** — a rich post-mortem instead of a silent `Killed`.
- ⚡ **Tiny & fast** — a single native Rust binary; near-zero idle cost.

---

## How it works — two-layer defense

### Layer 1 — The Wall (preventive, Linux)
Before your program starts, `mknight` installs a hard kernel memory cap with
`setrlimit(RLIMIT_AS)`. Once your program crosses the cap, the kernel makes
`malloc()`/`calloc()` return `NULL` — **the memory is never actually allocated**,
so the machine can't thrash. There's no monitoring race to lose.

> Bonus lesson: your program sees `malloc` return `NULL`, which is exactly what
> you should always check for.

### Layer 2 — The Watcher (reactive, cross-platform)
A high-frequency loop samples the child's resident memory (RSS) and enforces two
heuristics, killing the process the moment either trips:

| Heuristic | Trips when | Default |
|---|---|---|
| **A — Absolute limit** | RSS exceeds the ceiling | `1 GB` |
| **B — Explosive velocity** | memory grows faster than the threshold | `~500 MB / 100 ms` |

The two layers cover each other: untouched `malloc` grows virtual memory but not
RSS (Linux overcommit), and Layer 1 caps virtual address space; Layer 2 catches
hot, fast-growing leaks and produces the analytics.

---

## Install

Requires a [Rust toolchain](https://rustup.rs/) (stable).

```bash
# Once published to crates.io:
cargo install mknight

# Or build from source:
git clone https://github.com/FranciscoJBrito/mknight
cd mknight
cargo build --release        # binary at ./target/release/mknight
cargo install --path .       # or install it on your PATH
```

On **Linux** (x86_64), install the prebuilt binary — no Rust toolchain needed:

```bash
curl -fsSL https://raw.githubusercontent.com/FranciscoJBrito/mknight/main/install.sh | sh
```

It installs to `~/.local/bin` (override with `MKNIGHT_INSTALL_DIR`). You can also
download the binary manually from the
[Releases](https://github.com/FranciscoJBrito/mknight/releases) page.

### Adding `~/.local/bin` to your PATH

If the installer warns that `~/.local/bin` is not on your `PATH`, add it. Not sure
which shell you use? Run `echo $SHELL`, then pick the matching line:

```bash
# bash — appends to ~/.bashrc
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc

# zsh — appends to ~/.zshrc
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc
```

```fish
# fish — manages PATH its own way
fish_add_path ~/.local/bin
```

Then open a new terminal (or run the `source` line above) and `mknight --help`
should work.

## Updating

`mknight` doesn't auto-update. To get the latest version, re-run the same method
you installed with:

| Installed via | Update with |
|---|---|
| crates.io | `cargo install mknight` (upgrades to the latest; add `--force` to reinstall the same version) |
| `cargo-binstall` | `cargo binstall mknight` |
| Linux `curl` script | re-run `curl -fsSL https://raw.githubusercontent.com/FranciscoJBrito/mknight/main/install.sh \| sh` (it overwrites the binary in place) |
| Prebuilt binary (manual) | download the new one from [Releases](https://github.com/FranciscoJBrito/mknight/releases) and replace it |
| Built from source | `git pull && cargo install --path .` |

Check your current version with `mknight --version`. Tip: stick to one install
method to avoid having two copies on your `PATH`.

## Usage

```bash
# Basic — smart defaults (max 1 GB, or explosive growth)
mknight ./my_c_program

# Custom absolute ceiling
mknight --max-ram 500MB ./my_c_program

# Forward arguments to your program (everything after the program name)
mknight ./exercise arg1 arg2

# Watch and report, but never kill (build trust / debugging)
mknight --report-only ./my_c_program
```

### Options

| Flag | Default | Description |
|---|---|---|
| `--max-ram <SIZE>` | `1GB` | Absolute memory ceiling. Also sets `RLIMIT_AS` on Linux. |
| `--max-velocity <SIZE>` | `~500MB/100ms` | Explosive-growth threshold, per second (e.g. `5GB` = 5 GB/s). |
| `--interval <MS>` | `50` | Memory sampling interval, in milliseconds. |
| `--report-only` | off | Monitor and report, but never kill the child. |
| `--no-rlimit` | off | Skip the Layer 1 `setrlimit` wall (use Layer 2 only). |

Sizes use binary multipliers and accept `B`, `KB`/`K`, `MB`/`M`, `GB`/`G`
(case-insensitive): `1MB` = 1024 × 1024 bytes.

### Exit codes

- `137` — `mknight` killed the child for violating a safety policy (128 + SIGKILL).
- Otherwise — the child's own exit code (or `128 + signal` if it crashed).

---

## Try it

The repo ships two test programs under `examples/`:

```bash
# Bounded grower — safe to run anywhere; great for seeing the monitor live:
gcc -O2 examples/grow.c -o examples/grow
mknight --max-ram 200MB examples/grow 1000   # killed at the 200 MB ceiling

# Deliberate unbounded leaker — best run on Linux, where the wall makes
# malloc() return NULL cleanly:
gcc examples/leak.c -o examples/leak
mknight --max-ram 200MB examples/leak
```

---

## Platform support

| Platform | Layer 1 (Wall) | Layer 2 (Watcher) | Status |
|---|---|---|---|
| **Linux** | ✅ `setrlimit` | ✅ | Primary, fully supported |
| **macOS** | — (no-op) | ✅ best-effort | Supported (dev/test) |
| **Windows** | — | — | Planned (Job Objects) |

### macOS note
On macOS, per-process memory readings reflect `resident_size`, which the OS keeps
low by compressing and paging out cold pages. An "allocate-and-forget" leak may
therefore never show high RSS, so Layer 2 reliably catches only leaks whose
working set stays *hot*. The accurate metric is `phys_footprint` (what Activity
Monitor shows) — planned for a future release. macOS also handles memory pressure
gracefully, so the freeze problem `mknight` targets is primarily a Linux concern,
where both layers work fully.

---

## Contributing

Issues and pull requests are welcome. Please keep `cargo build`, `cargo clippy`,
and `cargo test` clean, and gate OS-specific code behind `#[cfg(...)]` so all
supported platforms keep compiling.

## License

Licensed under the [MIT License](LICENSE).
