//! Национальный охранник `CGuardCountry2`, тип ИИ `101`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает приоритет игрока другой страны и особое правило минимальной
//! дистанции. Сон использует общую проверку подключённых игроков в девяти
//! областях. `OnIdle` не выполняет случайный шаг: он ставит строгую очередь
//! `ChangeSkill → Stand → SearchEnemy`. Реакция на урон немедленно повторяет
//! ту же поисковую политику.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\guardcountry2.cpp
// `OnIdle` сопоставлен с RVA 0x00208E10.
// COMPONENT_VARIANT_END: GameServer

//! Общая достигнутая политика типов ИИ `13/14/20` находится в
//! `guardcountry.rs`; отдельного состояния и отличающихся побочных эффектов
//! у достигнутого пути нет.
