//! `CNotDisappearAfterDead`, GameServer.exe + GameServer.pdb,
//! исходный owner `appserver/other states/notdisappearafterdead.cpp`.
//! Payload UndeadState принадлежит общей арене CMoveShape, lifecycle User/Sufferer
//! хранится у того же экземпляра. Constructor0x005D62A0 копирует 72 байта без
//! часов и оставляет started/item timestamp нулевыми; factory snapshot допустим
//! и для innerID0, поскольку outer Add проверяет его только после старых End.
//! Object Begin0x005D7910 требует nonnull Sufferer до базы, но не CPlayer RTTI.
//! Первичный Begin(self,self) читает один base clock, сохраняет фактическую
//! identity, не меняет item timestamp и до append выполняет loop1 visual Update0.
//! Подготовленный payload не копируется и не регистрируется через этот вызов;
//! caller после успеха связывает обе identity, а каталог арены сохраняет ресурс
//! loop1 без повторного пакета. Недостижимый отказ native allocator не эмулируется.
//! Visual0x005D79C0 использует actual Sufferer и только живой не-ended ресурс:
//! BFE03(type,id,56,innerID,remaining,0) либо BFE04(type,id,56,innerID), затем
//! base visual tail даже при NULL S/ended. Helper ниже только кодирует пакет;
//! keyed gates/End/restart находятся в CGame и не копируют владеющий payload.
//! Vtable0x0065E2AC End+1C →0x005FD420: optional visual Update1, свежий
//! Sufferer и RemoveState, без base End/записи ended. AI0x005D7C80 сначала
//! проверяет ненулевой keep одним clock и строгим wrapping deadline<now;
//! затем нулевой item timestamp получает started. При ненулевых трёх item
//! параметрах отдельный clock проверяет срок и ещё один ставит timestamp
//! перед use_item через actual Sufferer. Нулевой результат вызывает End.
//! use_item0x005D7B20 возвращает фактически удалённое количество, поэтому
//! частичное ненулевое списание не завершает состояние. Несовпадение количества
//! отправляет GS1147 через SendSystemInfo0x0042CD70: BF807(FFFFFFFF,CString).
//! Общий адаптер CGame ограничивает результат native sprintf в buffer256
//! безопасными 255 байтами; NULL goods-name заменяет пустой строкой, а неверный
//! non-player cast — отказом с нулём. Это границы безопасности для native UB,
//! а не подтверждённое поведение оригинала на некорректных данных.
//! Пустой локальный CSeekGoodsListener не потребляется; отдельный объект не нужен.
//! Destructor0x005D6310 только вызывает базовый destructor; в Rust единственная
//! арена освобождает payload и принадлежащий этому экземпляру visual через Drop.
//! GetRemainedTime0x005D6320 при keep0 не читает часы; иначе берёт одно значение
//! для deadline<=now и второе для вычитания. Serialize0x005D64F0 записывает
//! вычисленный остаток в живой keep, не меняя started/item timestamp; writer
//! получает этот же остаток без второго clock. Новый record содержит полный keep.
//! Unserialize0x005D6530 одним clock ставит оба timestamp и копирует все 72 байта,
//! включая innerID0. Shared-span codec сохраняет padding +2..3 и исходный
//! ненулевой BOOL-байт, а не нормализует загруженную запись при сохранении.
//! OnUpdateProperties (vtable 0x0065E2AC +0x24 →0x005D6580) получает
//! GetSufferer: NULL даёт 0, только player 400 получает формулу, прочие — 1.
//! Visual, End, IsEnded и часы в этой функции не участвуют. Типизированная
//! арифметика находится в CPlayer::apply_undead_state_properties; вызов
//! конкретного экземпляра выполняет общий property-проход states/state.rs.
//! Проценты используют low32 IMUL, затем unsigned /100 (0x005D65D6,
//! 0x005D67FA), включая wrapping negation отрицательных параметров.
//! Вызванные setters 0x0042ACF0..0x0042AE10 ограничивают каждую unsigned
//! сумму INT_MAX. Отрицательные direct/CON HP/DEF/INT MP сохраняют signed
//! WORD-сужение, а STR/DEX и INT resistance/element — signed DWORD floor1.
//! FILD/FMUL не заменены ранним округлением всего выражения в f32:
//! STR/DEX используют полные целые; CON(-) сохраняет f32 только для DEF,
//! CON/INT(+) percentage — для delta. INT(-) MP использует __ftol2/FISTP64
//! и младший WORD (old полный, new f32), прочие производные INT(-) — f32.
//! Absolute INT(+) повторно проецирует полное новое INT после native cap.
//! Временный negation самого payload восстанавливается до возврата; между
//! этими записями только чистые player getters/setters, поэтому Rust считает
//! тот же результат без промежуточной мутации живого состояния.
//! Координатный Begin0x005D6360 и Begin с дополнительным типом цели
//! 0x005D6420 ещё не перенесены и остаются RAW-комментариями.

use crate::gameserver::appserver::moveshape::UndeadState;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) fn undead_state_visual_message(
    target: ShapeIdentity,
    state: &UndeadState,
    now: Option<&mut dyn FnMut() -> u32>,
) -> CMessage {
    let mut message = CMessage::new(if now.is_some() { 0x0b_fe03 } else { 0x0b_fe04 });
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_ulong(0x38);
    message.add_ulong(state.state_id());
    if let Some(now) = now {
        message.add_ulong(state.client_state_time(now));
        message.add_ulong(0);
    }
    message
}

pub(crate) fn begin_primary_undead_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    state: &mut UndeadState,
    now: &mut dyn FnMut() -> u32,
) -> Option<(i32, ShapeIdentity)> {
    resolve_state_move_shape(game, region_id, holder)?;
    state.begin_primary_at(now());
    let shape = resolve_state_move_shape(game, region_id, holder)?.shape();
    let participant = (
        shape.get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() },
    );
    let message = undead_state_visual_message(participant.1, state, Some(now));
    let _ = game.send_move_shape_around(participant.0, participant.1, &message);
    Some(participant)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp




// ============================================================================
// FUNCTION: CNotDisappearAfterDead::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:760
// RVA: 0x001D6360
// ADDRESS: 005d6360
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNotDisappearAfterDead::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\notdisappearafterdead.cpp:778
// RVA: 0x001D6420
// ADDRESS: 005d6420
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
