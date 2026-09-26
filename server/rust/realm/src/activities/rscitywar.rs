//! Пустой DB-адаптер `CRsCityWar` WorldServer, перенесённый в Realm
//! `activities/` рядом с расписанием `CAttackCitySys` (`attackcitysys`).
//!
//! Источник контракта — `worldserver.exe` и `worldserver.pdb`. Concrete-класс
//! содержит только lifetime базового `CMyAdoBase`; вызываемых DB-методов и
//! отдельной City War transaction phase у него нет. Сохранение военных данных
//! принадлежит другим DB-owner-ам.
//!
//! Rust не вводит фиктивный City War API: обычное владение и `Drop` заменяют
//! vtable/RTTI, COM cleanup и deleting-destructor пустого адаптера. Факт
//! создания фиксируют startup-дискриминант `WorldGameDatabaseOwner::RsCityWar`
//! (`app/world_runtime`) и bool-маркер переходного runtime старого пакета.
