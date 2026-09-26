//! Пустой DB-адаптер `CRsCityWar` WorldServer рядом с расписанием
//! `CAttackCitySys` (`attackcitysys`).
//!
//! Concrete-класс содержит только lifetime базового `CMyAdoBase`; вызываемых
//! DB-методов и отдельной City War transaction phase у него нет. Сохранение
//! военных данных принадлежит другим DB-owner-ам.
//!
//! Rust не вводит фиктивный City War API: обычное владение и `Drop` заменяют
//! vtable/RTTI, COM cleanup и deleting-destructor пустого адаптера. Факт
//! создания фиксирует startup-дискриминант `WorldGameDatabaseOwner::RsCityWar`
//! (`crate::app::world_runtime`).
