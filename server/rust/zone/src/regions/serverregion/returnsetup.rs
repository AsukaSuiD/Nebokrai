//! Return-setup семейство `CServerRegion`: данные игрока/окна возврата и
//! fallback-цепочка exact `GetReturnPoint` (`0x000814F0`). Исходный владелец
//! — `appserver/serverregion.h/.cpp`; точная пара `gameserver.exe` +
//! `GameServer/GameServer.pdb`.
//!
//! `GetReturnPoint` при null-игроке zero-write всех шести выходов сохраняет
//! 0-инициализацию результата; иначе локальный `m_stSetup` (`+0x170..+0x18C`,
//! восемь signed DWORD) имеет приоритет при `use_return != 0`, а иначе
//! выполняется fallback в три mutating country-default карты
//! `CCountryParam::main_return_point`. `DoesRecallWhenLost` (`0x0007BAC0`)
//! читает `+0x184`. Constructor не записывает `m_stSetup`: до доказанного
//! decoder/writer-а setup остаётся отдельной typed-границей
//! `ServerReturnSetupBlock`. Машинная сверка всего тела подтверждает:
//! null-ветвь — инлайн-порт базового `CRegion::GetReturnPoint` (RVA
//! `0x000F0280`, тот же zero-write шести выходов); use_return ветвь пишет
//! direction константой `-1`; fallback — три mutating карты
//! (`operator[]` семантика `main_return_point`); `does_recall` `+0x184` и
//! `use_return` `+0x18C` machine-verified. Поле `+0x188`
//! (`move_monster_when_refeash`) — INFERRED (ни одна из функций его не
//! читает); остальные 7 из 8 полей machine-verified. Ctor-claim
//! «не записывает setup» подтверждён телами `??0CServerRegion`/`??0CRegion`
//! (косвенный вызов — `timeGetTime()`, не setup).

use crate::content::countryparam::CCountryParam;
use crate::regions::region::RegionReturnPoint;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServerReturnPlayer {
    pub id: i32,
    pub country: u8,
    pub faction_id: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServerReturnSetup {
    pub region_id: i32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub does_recall_when_lost: i32,
    pub move_monster_when_refeash: i32,
    pub use_return: i32,
}

/// BLOCKED_MISSING_FACT: constructor не записывает `m_stSetup`; безопасный
/// результат возможен только после доказанного decoder/writer-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerReturnSetupBlock;

/// Тело exact `GetReturnPoint` без virtual-каркаса переходного владельца:
/// null-игрок получает zero-write результат базового `CRegion`, setup-окно с
/// `use_return != 0` побеждает country fallback, setup-направление фиксировано
/// `-1`.
pub fn get_return_point(
    setup: Option<ServerReturnSetup>,
    player: Option<ServerReturnPlayer>,
    country_param: &mut CCountryParam,
) -> Result<RegionReturnPoint, ServerReturnSetupBlock> {
    let Some(player) = player else {
        return Ok(RegionReturnPoint::default());
    };
    let setup = setup.ok_or(ServerReturnSetupBlock)?;
    if setup.use_return != 0 {
        return Ok(RegionReturnPoint {
            region_id: setup.region_id,
            left: setup.left,
            top: setup.top,
            right: setup.right,
            bottom: setup.bottom,
            direction: -1,
        });
    }

    let main = country_param.main_return_point(player.country);
    Ok(RegionReturnPoint {
        region_id: main.region_id,
        left: main.rect.left,
        top: main.rect.top,
        right: main.rect.right,
        bottom: main.rect.bottom,
        direction: main.direction,
    })
}

/// Тело exact `DoesRecallWhenLost` (`0x0007BAC0`): чтение `m_stSetup +0x184`
/// после той же typed-границы доказанного setup.
pub fn does_recall_when_lost(
    setup: Option<ServerReturnSetup>,
) -> Result<i32, ServerReturnSetupBlock> {
    setup
        .map(|setup| setup.does_recall_when_lost)
        .ok_or(ServerReturnSetupBlock)
}
