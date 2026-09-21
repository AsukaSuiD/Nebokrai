[English](README.md) | [Русский](README.ru.md)

# Nebokrai

**An independent reconstruction of a legacy MMORPG server platform, implemented in Rust for Linux.**

Nebokrai studies and rebuilds the server-side behavior of *Поднебесье* (Miracle). Its purpose is software preservation and interoperability with the legacy client: recovering message formats, gameplay rules, persistence semantics, and the order of observable effects in a maintainable implementation.

The project combines server engineering with documented reconstruction. Historical symbols and binary evidence help identify behavior; uncertainty is recorded explicitly rather than filled with plausible substitutes. This is an independent modern implementation effort, not a release of the original server source code or a claim of complete compatibility.

## Architecture

The primary implementation is one Rust package with six server executables:

| Service | Responsibility |
| --- | --- |
| **Auth** | Account verification for the supported authentication route. |
| **Login** | Client entry, authentication routing, and world selection. |
| **World** | Character loading and persistence, world state, and routing to Game. |
| **Game** | Regional simulation, movement, combat, items, NPCs, and client gameplay messages. |
| **Billing** | Account balances and purchase/exchange operations. |
| **Misc** | Auction processing and its connection to World. |

Game owns live regional state; World owns character persistence. The services exchange legacy TCP messages. Tokio provides networking and task execution, while Tiberius connects to the existing Microsoft SQL Server data model. Replacing the infrastructure does not make message ordering, integer behavior, timing, or partial database effects interchangeable.

See the [state ownership guide](docs/architecture/state-ownership.md), [protocol documentation](docs/protocol/README.md), and [architecture decisions](docs/decisions/README.md).

## Current status

Nebokrai is a reconstruction in progress, not a production-ready game server. The [project status](docs/status/audit.md) is the authoritative record of builds, runtime observations, and client checks, including the commits and limitations of each observation.

As recorded there on 21 September 2026:

- The six Rust executables passed Linux compilation and were run together in a local development setup.
- The original client reached the character list; a subsequent selection loaded a character and registered a player in Game.
- A stable, playable session has **not** been demonstrated. Later protocol and movement fixes still require the relevant client scenarios to be repeated.
- Persistence across gameplay sessions, region transitions, auction and billing operations, complete SQL behavior, and sustained performance are not established by that startup check.

Individual contracts carry `VERIFIED`, `INFERRED`, `PARTIAL`, or `UNKNOWN` evidence labels. A verified original behavior does not imply that its implementation has passed a client scenario. The [evidence rules](docs/reconstruction/evidence-and-contracts.md) explain that distinction.

## Getting started

Start with the [developer guide](docs/development.md) and [workspace map](docs/architecture/workspace.md). Extended technical documentation is currently **primarily in Russian**; it is intentionally preserved rather than replaced with an incomplete translation.

For a source check on Linux, install the toolchain specified by [rust-toolchain.toml](server/rust/rust-toolchain.toml), then run from the repository root:

```sh
cd server/rust
cargo check --locked --lib --bins
```

The [build guide](docs/operations/build.md) covers native requirements, binary builds, and the PowerShell/Docker Desktop workflow. The primary server targets Linux; native Windows compilation is not supported by the current process layer. Reading or editing documentation does not require builds or server launches.

Running the services additionally requires locally supplied configuration, compatible game resources, and database data that are not part of the source checkout. Each process reads from its working directory. A clone alone is **not a runnable game distribution**. See the [all-Rust local setup](docs/operations/rust-runtime.md); the separate [hybrid setup](docs/operations/hybrid-runtime.md) uses an original GameServer under Wine as a local reference.

## Repository layout

| Path | Contents |
| --- | --- |
| [`server/rust/`](server/rust/) | Primary Rust implementation and six binary entry points. |
| [`server/cpp/`](server/cpp/) | Separate C++ reconstruction pass; not part of the Rust build. |
| [`deploy/`](deploy/) | Build and local deployment tooling. |
| [`docs/`](docs/README.md) | Architecture, protocols, gameplay, evidence, operations, and status. |

`runtime/`, `original/`, `related/`, and local analysis directories are excluded from Git. Original EXE/DLL/PDB files, the game client, proprietary game assets, database backups, and private research captures are outside the intended source distribution. References to those files identify local evidence; they are not download links or redistribution permission.

Publication review has also identified residual RAW decompiler material in tracked source files. Its disposition and historical content must be resolved before publication; see publication readiness (локальный материал владельца). Do not interpret the intended distribution boundary as a certification of the entire Git history.

## Contributing and licensing

Read [CONTRIBUTING.md](CONTRIBUTING.md) for issue and pull request guidance, evidence requirements, and the policy against adding automated tests. Security reporting is described in [SECURITY.md](SECURITY.md). Please do not upload original binaries, game assets, live credentials, account data, or private captures to issues or pull requests.

**A project license has not yet been selected.** This preparation does not grant an open-source license or establish permission to redistribute third-party material. Licensing options (локальный материал владельца) records the decision still required from the owner, including future commercial licensing and contributions.

Nebokrai is not affiliated with, endorsed by, or supported by the original game's developers or publishers. Game names and other trademarks belong to their respective owners.
