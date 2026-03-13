# evosim

An evolution simulator where soft-body creatures learn to walk through genetic algorithms and physics simulation, written in Rust.

## Project Structure

```
evosim/
├── Cargo.toml                  Workspace root
├── README.md
└── crates/
    ├── evosim-core/            Pure physics engine (no graphics)
    │   └── src/
    │       ├── lib.rs          Re-exports all public types
    │       ├── particle.rs     Point-mass particle (Verlet integration)
    │       ├── muscle.rs       Spring connection between particles
    │       ├── creature.rs     Soft-body creature (particles + muscles)
    │       ├── world.rs        Top-level simulation state
    │       └── constants.rs    Physics constants (gravity, damping, etc.)
    │
    ├── evosim-genetics/        Genome encoding, mutation & selection
    │   └── src/
    │       ├── lib.rs          Re-exports all public types
    │       ├── genome.rs       Variable-length float genome
    │       ├── mutation.rs     Per-gene mutation operators
    │       └── selection.rs    Tournament selection
    │
    ├── evosim-renderer/        Optional Bevy-based visualizer
    │   └── src/
    │       └── lib.rs          Bevy plugin stub
    │
    └── evosim-cli/             Main binary — orchestrates everything
        └── src/
            └── main.rs         Entry point with TOML config loading
```

## Crate Dependency Graph

```
evosim-cli
├── evosim-core
├── evosim-genetics
├── evosim-renderer
│   └── evosim-core
├── serde
└── toml

evosim-core        → glam, rayon
evosim-genetics    → rand, rand_distr
evosim-renderer    → bevy, evosim-core
```

## Building

```bash
cargo build
```

To build without the renderer (faster compile times):

```bash
cargo build -p evosim-core -p evosim-genetics
```

## Running

```bash
cargo run -p evosim-cli
```

## License

MIT
