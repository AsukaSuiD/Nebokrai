//! Мировые рейтинги Realm. Первый кирпич — honor-eliminator индекс;
//! `CPlayerRanks`/`CHonorRanks` остаются у `characters` и в этот цикл не
//! мигрируют.

pub mod honoreliminators; // honor-eliminator индекс мира: убитый игрок → его убийцы (ветви 0x5FD0C/0x5FD0D).
