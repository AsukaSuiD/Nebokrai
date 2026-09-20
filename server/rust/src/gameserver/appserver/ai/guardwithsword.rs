//! Ближний вариант городской охраны `CGuardWithSword`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает тот же приоритет преступника перед неохранным монстром и то же
//! правило минимальной дистанции навыка, что у `CGuardWithBow`. Реальный тип
//! ИИ `9` использует общий владелец поиска. Конструктор 0x0060E9F0 задаёт
//! координатам поста sentinel -1; первый OnIdle (0x0060EA10) сохраняет
//! позицию и ставит ChangeSkill(6) перед базовым OnIdle, даже если навык уже
//! выбран. Пост не фиксируется при преждевременном входе в бой.
//! OnLoseTarget (0x0060D020) и Tracing (0x0060D0E0) совпадают с городским
//! мечевым стражем: общий GuardStationState и обработчики cityguardwithsword
//! сохраняют возврат на пост без отмены cast/Move. Выбор этого семейства
//! централизован в MonsterAiKind::has_guard_station; selector преступников
//! не подменяется городским faction/union-поиском.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\guardwithsword.cpp
// COMPONENT_VARIANT_END: GameServer
