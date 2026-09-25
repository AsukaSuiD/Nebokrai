//! Блоки governance-контекста страны, вынесенные сюда заранее: сама
//! `CCountry` остаётся в старом `appworld/country/country.rs` до шага
//! переноса области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryGovernanceContextBlock {
    FactionMasterLookup,
    PlayerFactionLookup,
    UnionLookup,
    OwnedCityMutation,
    FactionDemise,
}
