[English](README.md) | [Русский](README.ru.md)

# Nebokrai

**An independent Rust/Linux reconstruction of a closed-source MMORPG server platform.**

Nebokrai rebuilds the server-side behavior of *Поднебесье* (Miracle), with compatibility with its legacy client as the goal. It is a new implementation, not the original server source code.

Preservation means documenting how the game worked: its messages, gameplay rules, persistence, and observable behavior. Interoperability means making the replacement services communicate correctly with the existing client and with each other. Both require evidence; plausible behavior is not enough.

The available Git history begins on **18 August 2026**. The [project history](docs/overview/history.md) follows the recorded work from binary research to six Rust services. Beyond this game, Nebokrai offers concrete material for studying MMO backends, protocol reconstruction, state ownership, and Rust server development.

## Architecture

The workspace contains the existing server package and the Shared, Realm, and Zone libraries. Realm prepares script resources; Zone owns their received texts and replacement. The server package still builds six executables:

| Service | Responsibility |
| --- | --- |
| **Auth** | Account verification on the authentication route that uses Auth. |
| **Login** | Client entry, authentication routing, and world selection. |
| **World** | Character loading and persistence, world state, and routing to Game. |
| **Game** | Live regional simulation: movement, combat, items, NPCs, and gameplay messages. |
| **Billing** | Account balances and purchase/exchange operations. |
| **Misc** | Auctions and their interaction with World. |

Game owns live regional state; World handles character persistence. Tokio provides networking and task execution, and Tiberius connects to Microsoft SQL Server. Modern infrastructure must still preserve significant message ordering, timing, and database effects. See the illustrated [architecture tour](docs/overview/architecture.md) and [player login journey](docs/overview/player-journey.md).

## Current status

As documented on **21 September 2026**:

- All six Rust services compiled and ran together in a local Linux development setup.
- The legacy client reached the world and the owner was able to move a character, although movement was visibly delayed.
- A later Game-loop correction reduced the measured release AI pass to roughly the 80 ms tick budget; a clean release restart reached 549 regions and a healthy client listener.
- **A stable, playable session has not been demonstrated.** The same client scenario still needs to be repeated on that corrected release build.
- Full persistence across sessions, region transitions, billing, auctions, and sustained performance remain unproven.

The current work is component refactoring with technical debt and defects addressed in each affected area; see the [roadmap](docs/overview/roadmap.md). The [status overview](docs/overview/status.md) separates implementation from observed behavior and links to the detailed evidence.

## Reconstruction methodology

Research starts with an identified executable and matching PDB symbols. Ghidra helps locate candidate behavior; machine instructions, the peer implementation, and reproducible runtime observations establish what can actually be claimed. Decompiler output alone is not proof.

Contracts distinguish `VERIFIED`, `PARTIAL`, `INFERRED`, and `UNKNOWN`. Confirming original behavior does not prove that the new implementation works with the client. Read the [methodology](docs/reconstruction/overview.md) and [region-entry case study](docs/reconstruction/case-study-region-entry.md).

## Getting started

The [documentation portal](docs/README.md) connects history, architecture, status, and subsystem guides. Start development with the [developer guide](docs/development.md) and [workspace map](docs/architecture/workspace.md). Detailed documentation is primarily in Russian; key overviews have English summaries. Russian remains the working language, including commit messages.

On Linux, install the toolchain in [rust-toolchain.toml](server/rust/rust-toolchain.toml), then run from the repository root:

```sh
cd server/rust
cargo check --locked --workspace --lib --bins
```

The [build guide](docs/operations/build.md) covers dependencies and the PowerShell/Docker workflow. Native Windows compilation is not currently supported. Running the services requires separately supplied local configuration, compatible resources, and database data: a clone is not a runnable game distribution.

Source layout: [Rust implementation](server/rust/), [separate C++ reconstruction](server/cpp/), [deployment tools](deploy/), and [documentation](docs/README.md).

## Materials, contributions, and licensing

Original EXE/DLL/PDB files, the client, game assets, database backups, private captures, and full decompiler output are outside the intended source distribution. Public documentation contains authored findings and provenance metadata; references to local evidence do not provide downloads or redistribution rights.

[CONTRIBUTING.md](CONTRIBUTING.md) explains the fork/branch → pull request → review → main workflow. Do not attach proprietary materials, credentials, or user data to issues or PRs. See [SECURITY.md](SECURITY.md) for the current reporting policy.

Nebokrai's own source code and documentation are licensed under the [GNU Affero General Public License, version 3 only](LICENSE) (`AGPL-3.0-only`). Dependencies retain their respective licenses; see [third-party components and credits](THIRD_PARTY_NOTICES.md). This license does not cover the original game materials or private research corpus, and does not certify third-party rights.

Nebokrai is not affiliated with or endorsed by the original game's developers or publishers. Game names and trademarks belong to their respective owners.
