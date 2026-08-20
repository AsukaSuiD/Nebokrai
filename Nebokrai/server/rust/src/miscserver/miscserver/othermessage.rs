//! Свободный handler `OnOtherMsg` из `miscserver/othermessage.cpp`.
//!
//! Статус владельца: `IMPLEMENTED`; ширина единственного response-поля имеет
//! статус `VERIFIED_DISASSEMBLY`.
//!
//! Точная пара: `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`;
//! SHA-256 EXE
//! `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65`,
//! SHA-256 PDB
//! `ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7`.
//! Исходный путь PDB:
//! `h:\fengyun\fy_russia\src\server\miscserver\miscserver\othermessage.cpp`.
//! `OnOtherMsg` — RVA `0x00003CC0`.
//!
//! Handler знает только два полных opcode. `0x0007F809` при существующем
//! `CMyNetClient` строит `0x0005FA0A`, а `0x0007F80B` без предварительной
//! проверки client строит `0x0005FA0C`. Оба ответа содержат один нулевой
//! 32-битный payload и отправляются без приоритета; результат `Send` исходник
//! игнорировал. При отсутствии client первая ветвь завершается до создания
//! сообщения, тогда как вторая всё равно вызывает `Send` и получает старый
//! нулевой результат. Остальные opcode ничего не делают.
//!
//! Ghidra-псевдокод потерял перегрузку `CBaseMessage::Add(..., 0)`. Точечное
//! дизассемблирование exact EXE подтвердило: call sites `0x00403D12` и
//! `0x00403D4F` вызывают RVA `0x00010F20`, который копирует четыре байта и
//! увеличивает длину на четыре. Нулевой signed/unsigned bit-pattern одинаков;
//! Rust использует существующий `add_long(0)` и не назначает полю доменное имя.
//!
//! Nullable входной `CMessage*` заменён обязательной ссылкой: `CMessage::Run`
//! вызывает handler только для живого owned сообщения. Nullable global
//! `s_pNetClient` остаётся `Option<&dyn MessageSender>`. SEH, stack-local
//! destructors и ручной выбор удаляемого `CMessage*` заменены владением и
//! `Drop`; иных функций или неразрешённых блоков в этом owner-файле нет.

use crate::nets::netmisc::message::{CMessage, MessageSender, SendMessageError};

const WORLD_STATUS_REQUEST: i32 = 0x0007_F809;
const WORLD_STATUS_RESPONSE: i32 = 0x0005_FA0A;
const WORLD_SYNC_REQUEST: i32 = 0x0007_F80B;
const WORLD_SYNC_RESPONSE: i32 = 0x0005_FA0C;

/// Наблюдаемый итог одного вызова исходного `OnOtherMsg`.
#[derive(Debug)]
pub(crate) enum OtherMessageOutcome {
    /// Opcode не входил в две известные ветви handler-а.
    Unhandled,
    /// `0x7F809` завершился до создания ответа из-за отсутствующего client.
    MissingClient,
    /// Ответ создан, а исходно игнорировавшийся результат отправки сохранён.
    Response {
        message_type: i32,
        send: Result<i32, SendMessageError>,
    },
}

/// Обрабатывает две доказанные служебные ветви World-to-Misc.
pub(crate) fn on_other_msg(
    message: &CMessage,
    sender: Option<&dyn MessageSender>,
) -> OtherMessageOutcome {
    let response_type = match message.message_type() {
        WORLD_STATUS_REQUEST if sender.is_none() => return OtherMessageOutcome::MissingClient,
        WORLD_STATUS_REQUEST => WORLD_STATUS_RESPONSE,
        WORLD_SYNC_REQUEST => WORLD_SYNC_RESPONSE,
        _ => return OtherMessageOutcome::Unhandled,
    };

    let mut response = CMessage::new(response_type);
    response.base_mut().add_long(0);
    OtherMessageOutcome::Response {
        message_type: response_type,
        send: response.send(sender, false),
    }
}
