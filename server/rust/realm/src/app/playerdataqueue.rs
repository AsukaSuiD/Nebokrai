//! Стадия MainLoop, воспроизводящая FIFO загруженных игроков
//! (`CGame::ProcessPlayerDataQueue`), подтверждённая `Nworldserver.exe`
//! (SHA-256 `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`)
//! и парным `WorldServer.pdb` (RSDS `289F1FB3-96A0-4FF4-8B5D-1FD17B50B751`,
//! age 1): pubs `?ProcessPlayerDataQueue@CGame@@QAEXXZ`,
//! `?GetSize@CPlayerDataQueue@@QAEIXZ`,
//! `?PopPlayerData@CPlayerDataQueue@@QAEPAUtagPlayerDataQueue@@XZ`,
//! `?PushPlayerData@CPlayerDataQueue@@QAE_NPAUtagPlayerDataQueue@@@Z`.
//!
//! Сама FIFO живёт в [`crate::characters::playerdataqueue`]; здесь её
//! MainLoop-потребитель. За проход обрабатывается не более одной non-null
//! записи начального snapshot: null-pop уменьшает сохранённый snapshot и
//! повторяет pop, а извлечённая запись уходит в loaded-queue маршрут владельца
//! игры (шов [`WorldPlayerQueueGameView`]) и безусловно завершает вызов.
//! Стадия накапливает profile clock между теми же tick-вызовами, что исходный
//! MainLoop; состояния clock/profile этой волной остаются в старом пакете,
//! handler получает их поля по ссылке и пишет в исходных позициях
//! относительно tick-вызовов.

use crate::app::world_game_view::{
    WorldPlayerQueueGameView, WorldProcessPlayerDataQueueBlock, WorldProcessPlayerDataQueueError,
    WorldProcessPlayerDataQueueOutcome,
};
use crate::characters::player::CPlayer;
use crate::characters::playerdataqueue::CPlayerDataQueue;

/// Отчёт одной queue-стадии MainLoop: при блокирующем дефекте — тот же
/// producer-error, при завершении — исход прохода и времена двух закрывающих
/// тиков вместе с накопленным profile time. Тип перенесён из `game.rs` вместе
/// со stage-handler-ом; старый пакет реэкспортирует.
#[derive(Debug, Eq, PartialEq)]
pub enum WorldMainLoopPlayerDataQueueStageReport {
    Complete {
        outcome: WorldProcessPlayerDataQueueOutcome,
        finished_at_ms: u32,
        elapsed_ms: u32,
        accumulated_time_ms: u32,
        next_stage_started_at_ms: u32,
    },
    Blocked {
        error: WorldProcessPlayerDataQueueError,
    },
}

/// Обрабатывает не более одного non-null record-а из начального snapshot.
///
/// Null-pop не завершает метод: он уменьшает только сохранённый snapshot и
/// повторяет pop. Любая фактически извлечённая запись проходит ровно одну
/// reject либо success цепочку и затем безусловно завершает вызов.
pub fn process_player_data_queue<G: WorldPlayerQueueGameView>(
    game: &mut G,
    organizing_ctrl: &mut G::OrganizingContext,
    player_data_queue: &CPlayerDataQueue<CPlayer>,
    get_tick: &mut dyn FnMut() -> u32,
) -> Result<WorldProcessPlayerDataQueueOutcome, WorldProcessPlayerDataQueueError> {
    let initial_size = player_data_queue.get_size();
    let mut remaining = initial_size;
    let mut null_pops = 0_u32;

    while remaining != 0 {
        let Some(mut entry) = player_data_queue.pop_player_data() else {
            remaining = remaining.wrapping_sub(1);
            null_pops = null_pops.wrapping_add(1);
            continue;
        };

        let queue_player_id = entry.player_id();
        let client_ip = entry.client_ip();
        let Some(cdkey) = entry.cdkey().map(<[u8]>::to_vec) else {
            return Err(WorldProcessPlayerDataQueueError {
                initial_size,
                null_pops,
                player_id: queue_player_id,
                block: WorldProcessPlayerDataQueueBlock::UnterminatedCdkey,
            });
        };
        let mut after_login_send = |_player: &mut CPlayer| {};
        return game.route_loaded_player(
            organizing_ctrl,
            initial_size,
            null_pops,
            queue_player_id,
            client_ip,
            &cdkey,
            entry.take_player(),
            &mut after_login_send,
            &mut *get_tick,
        );
    }

    Ok(WorldProcessPlayerDataQueueOutcome::NoRecord {
        initial_size,
        null_pops,
    })
}

/// Закрывает queue-стадию исходного MainLoop: проход очереди, затем два
/// tick-чтения с закрытием elapsed и переносом stage start. При дефекте
/// прохода ни один tick не тратится и ни одно profile clock поле не пишется,
/// как у исходной стадии.
pub fn run_main_loop_player_data_queue_stage<G: WorldPlayerQueueGameView>(
    game: &mut G,
    organizing_ctrl: &mut G::OrganizingContext,
    player_data_queue: &CPlayerDataQueue<CPlayer>,
    stage_started_at_ms: &mut u32,
    process_player_data_queue_time_ms: &mut u32,
    get_tick: &mut dyn FnMut() -> u32,
) -> WorldMainLoopPlayerDataQueueStageReport {
    let outcome = match process_player_data_queue(
        game,
        organizing_ctrl,
        player_data_queue,
        &mut *get_tick,
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            return WorldMainLoopPlayerDataQueueStageReport::Blocked { error };
        }
    };
    let finished_at_ms = get_tick();
    let elapsed_ms = finished_at_ms.wrapping_sub(*stage_started_at_ms);
    *process_player_data_queue_time_ms =
        process_player_data_queue_time_ms.wrapping_add(elapsed_ms);
    let next_stage_started_at_ms = get_tick();
    *stage_started_at_ms = next_stage_started_at_ms;
    WorldMainLoopPlayerDataQueueStageReport::Complete {
        outcome,
        finished_at_ms,
        elapsed_ms,
        accumulated_time_ms: *process_player_data_queue_time_ms,
        next_stage_started_at_ms,
    }
}
