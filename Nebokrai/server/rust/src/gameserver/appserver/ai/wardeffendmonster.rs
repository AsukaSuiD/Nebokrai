//! Поиск цели обороняющего монстра войны стран `WarDeffendMonster`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает поиск ближайшего живого противника по лагерю `CountryWarSys`.
//! Реальный тип ИИ `17` сохраняет порядок игроков перед питомцами и замену
//! предыдущей цели при равной `RealDistance`.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\wardeffendmonster.cpp
// COMPONENT_VARIANT_END: GameServer

//! Общий алгоритм семейства находится в `warattackmonster.rs`; отдельного
//! состояния и отличающихся побочных эффектов у достигнутого пути нет.
