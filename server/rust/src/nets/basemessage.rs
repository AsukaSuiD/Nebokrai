//! Базовый 16-байтовый wire-буфер сообщений, восстановленный из
//! `nets/basemessage.h` и `nets/basemessage.cpp`.
//!
//! Статус функций layout, курсора, `Update`, числовых и сырых `Add/Get`,
//! `AddEx`, строкового `Add`, обоих вариантов `GetStr`, `CGUID Add/Get` и
//! корректных RLE-потоков: `IMPLEMENTED`. `GetEx` и повреждённый RLE-marker
//! имеют локальные статусы `UNKNOWN` (исследовательский декомпилят хранится локально) либо `BLOCKED_MISSING_FACT` ниже.
//!
//! Точные варианты корпуса:
//! - `AuthServer/authserver.exe + AuthServer/authserver.pdb`, SHA-256
//!   `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15` /
//!   `26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5`;
//! - `BillingServer/billingserver.exe + BillingServer/billingserver.pdb`,
//!   `FA32E3C043CB49965686129696A4EB34B733ACA1D60CAF57D369F97D5E68FB19` /
//!   `F900CD0330BEFF32AC071B107AB653FD403CD18746896B3C0187C5751ACA0B21`;
//! - `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`,
//!   `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876` /
//!   `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`;
//! - `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`,
//!   `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65` /
//!   `ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7`;
//! - `GameServer/gameserver.exe + GameServer/GameServer.pdb`,
//!   `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E` /
//!   `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`;
//! - `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//!   `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1` /
//!   `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//!
//! Исходные пути PDB: `h:\fengyun\fy_russia\src\nets\basemessage.cpp`,
//! `d:\complite_version\fengyun_russia\trunk\nets\basemessage.cpp` и
//! `e:\svn\fengyun_russia_dev\nets\basemessage.{h,cpp}`.
//!
//! Существенные RVA реализованного слоя:
//! - конструктор: Auth `0x00012FB0`, Billing `0x0000D530`, Login
//!   `0x00066360`, Misc `0x00011100`, Game `0x00013520`, World `0x00023EE0`;
//! - числовые `Get`: Auth `0x000129B0..0x00012A10`, Billing `0x0000CF10`,
//!   Login `0x00065C10..0x00065C70`, Misc `0x00010CC0..0x00010CF0`, Game
//!   `0x00012BF0..0x00012CC0`, World `0x000236F0..0x000237C0`;
//! - сырые `Get`: Billing `0x0000CFA0`, Login `0x00065D00`, Misc
//!   `0x00010D80`, Game `0x00012D50`, World `0x00023850`;
//! - `CGUID Get/Add`: Billing `0x0000D030/0x0000D5B0`, Misc
//!   `0x00010E00/0x00011180`, Game `0x00012E50/0x000135A0`, World
//!   `0x000238A0/0x00023F60`;
//! - `AddEx`: Game `0x00013490`, World `0x00023E50`; inline `Update`: Game
//!   `0x00005060`, World `0x0001A990`.
//!
//! Во всех шести вариантах конструктор добавляет шестнадцать нулевых байт,
//! записывает little-endian Windows `long(16)` в первые четыре байта и ставит
//! курсор на offset `16`. Эти шестнадцать байт резервируются буквально:
//! `+4` доказан как `MsgType` конструкторами, `Run` и send-логами конкретных
//! `CMessage`. Слова `+8` и `+0xC` конструктор оставляет нулевыми, а receive
//! копирует буквально; иных записей в шести `message.rs` не найдено. До
//! появления факта они остаются opaque/reserved и здесь не получают имён.
//!
//! Числовые значения копировались как байты x86 в little-endian порядке.
//! Обычная нехватка данных возвращала ноль и не двигала курсор; сырой `Get`
//! возвращал `nullptr`. Rust API выражает эту потерю данных через `Option` или
//! `bool`, не смешивая корректный ноль с ошибкой. `char`, `unsigned char`,
//! `short`, `unsigned short`, Windows `long`/`unsigned long`, `__int64` и
//! `float` сохранены отдельными методами с точными размерами и signedness.
//!
//! `Vec<u8>` заменяет внутренности `std::vector<unsigned char>` и allocator.
//! После каждого добавления первые четыре байта обновляются фактической длиной,
//! что совпадает с исходным `Add` и inline `Update` на достижимых размерах.
//! Срезы Rust заменяют пары nullable pointer + signed length: отрицательная
//! длина и чтение из `nullptr` не являются состояниями этого API. `CStr`
//! сохраняет исходный `lstrlenA`-контракт: `None` ничего не добавляет, реальная
//! строка добавляется вместе с единственным завершающим нулём без требования
//! UTF-8.
//!
//! Маркер GUID также сохранён буквально: `0` означает `GUID_INVALID`, любое
//! ненулевое значение заставляет прочитать следующие 16 байт; исходный writer
//! использовал `0x10`. `Option<CGuid>` соответствует старой паре
//! `bool + out-param`: `None` объединяет исходный invalid GUID и нехватку байт,
//! как это делал возвращаемый `false`, а курсор после уже прочитанного маркера
//! не откатывается.
//!
//! `DoRLE` присутствует в Login RVA `0x000659A0` и Game RVA `0x00012930`.
//! `DecodeRLE_SAFE` присутствует в Billing `0x0000CE30`, Login `0x00065B30`,
//! Misc `0x00010C00`, Game `0x00012AF0` и World `0x000235C0`. Корректный wire
//! совпадает во всех вариантах: `0x00..=0xF7` — literal; одиночный байт
//! `0xF8..=0xFF` экранируется парой `0xF8, byte`; пара `marker, byte` для marker
//! `0xF8..=0xFF` означает повтор byte от одного до восьми раз. Длинная серия
//! дробится на `0xFF, byte` и последнюю пару. Rust-функции возвращают владеющий
//! `Vec<u8>` вместо записи в глобальный scratch-buffer.
//!
//! Аудит send-path доказал время жизни scratch-указателя. `CServer::SendAll` и
//! `SendBySocketID` копируют вход до возврата (Billing RVA `0x000078D0` /
//! `0x00007950`, Login `0x000664C0` / `0x00066540`, Game `0x00014E10` /
//! `0x00014E90`, World `0x00024080` / `0x00024100`); `CClient::SendToServer`
//! делает то же в Login `0x0006B0F0`, Misc `0x000113F0`, Game `0x000191D0` и
//! World `0x00028960`. Поэтому локальный `Vec` безопасно заменяет lifetime
//! общей памяти: конкретный `CMessage` может построить буфер и передать его
//! owned/copying send-владельцу. `Initial`, `Release`, `DoTemptBuffer` и сами
//! глобальные буферы отдельного Rust-аналога не получают.
//!
//! Это доказательство не отменяет автоматически всю окружающую
//! `m_CSTemptBuffer`: часть send-методов держит её на последовательности
//! build/CRC/send, а часть аналогичных методов не захватывает вообще. Сохранять
//! ли сериализацию между параллельными отправлениями, обязан решить каждый
//! конкретный `CMessage` после проверки своей потоковой достижимости. До этого
//! здесь утверждаются только независимость памяти и порядок операций одного
//! вызова, а не межпоточный порядок send.
//!
//! Деструктор, `std::vector`, allocator, deleting-destructor, `$E/$L` cleanup,
//! ошибочно попавшие сюда `CMsgQueue` и специализации `std::deque` удалены как
//! compiler/STL noise либо уже принадлежат `nets/msgqueue.rs`. Их существенный
//! эффект выражен `Vec`, владением и `Drop` Rust. Полный сырой экспорт остаётся
//! в истории Git и воспроизводится из неизменяемых архивных EXE/PDB.

use std::ffi::CStr;

use zerocopy::byteorder::little_endian::U32;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use nebokrai_shared::values::CGuid;

const HEADER_LEN: usize = 16;
const RLE_MARKER_BASE: u8 = 0xF7;
const RLE_FIRST_MARKER: u8 = 0xF8;
const RLE_MAX_RUN: usize = 8;

/// Полностью подтверждённый фиксированный заголовок общих сообщений.
///
/// Два последних слова остаются непрозрачными и при получении или создании
/// копируются без интерпретации. Порядок байтов выражен типами `zerocopy`; все
/// битовые значения `u32` допустимы, а размер не зависит от ручной нарезки
/// диапазонов.
#[repr(C)]
#[derive(Clone, Copy, FromBytes, Immutable, IntoBytes, KnownLayout)]
struct LegacyMessageHeader {
    length: U32,
    message_type: U32,
    reserved: [U32; 2],
}

/// Ошибка кодирования на границе, для которой исходное поведение не определено
/// безопасным контрактом.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum RleEncodeError {
    /// Исходный `DoRLE` разыменовывал первый байт даже при нулевой длине.
    EmptyInputReactionUnknown,
}

/// Ошибка декодирования RLE-потока.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum RleDecodeError {
    /// Нулевой результат исходный вызывающий трактовал как отказ создания.
    EmptyInput,
    /// Исходная строгая проверка требовала оставить минимум один свободный байт.
    OutputCapacityReached,
    /// Реакция оригинала на marker без следующего value-byte пока не доказана.
    TrailingMarkerReactionUnknown,
}

/// Кодирует непустой набор байт точным legacy RLE-форматом.
///
/// Нулевой ввод возвращает отдельную неизвестную границу: исходная функция
/// читала первый байт вне переданного диапазона, и Rust не назначает этому UB
/// придуманное wire-представление.
pub(crate) fn encode_rle(input: &[u8]) -> Result<Vec<u8>, RleEncodeError> {
    let Some((&first, tail)) = input.split_first() else {
        return Err(RleEncodeError::EmptyInputReactionUnknown);
    };

    let mut output = Vec::with_capacity(input.len().saturating_mul(2));
    let mut run_byte = first;
    let mut run_length = 1;

    for &byte in tail {
        if byte == run_byte {
            run_length += 1;
        } else {
            append_rle_run(&mut output, run_byte, run_length);
            run_byte = byte;
            run_length = 1;
        }
    }
    append_rle_run(&mut output, run_byte, run_length);
    Ok(output)
}

/// Декодирует legacy RLE при исходном строгом ограничении output capacity.
///
/// Корректный результат всегда короче `output_capacity`. Marker последним
/// байтом возвращает локальную неизвестность, а не автоматически выбранный
/// fail-closed ответ оригинального процесса.
pub(crate) fn decode_rle(input: &[u8], output_capacity: usize) -> Result<Vec<u8>, RleDecodeError> {
    if input.is_empty() {
        return Err(RleDecodeError::EmptyInput);
    }

    let mut output = Vec::new();
    let mut index = 0;
    while index < input.len() {
        let marker = input[index];
        if marker < RLE_FIRST_MARKER {
            if output.len() + 1 >= output_capacity {
                return Err(RleDecodeError::OutputCapacityReached);
            }
            output.push(marker);
            index += 1;
            continue;
        }

        let Some(&value) = input.get(index + 1) else {
            // BLOCKED_MISSING_FACT: Billing RVA 0x0000CE30, Login 0x00065B30,
            // Misc 0x00010C00, Game 0x00012AF0 и World 0x000235C0 читают
            // `param_1[i + 1]` без проверки. Достижимость и наблюдаемая реакция
            // процесса на последний marker без value-byte пока не доказаны.
            return Err(RleDecodeError::TrailingMarkerReactionUnknown);
        };
        let run_length = usize::from(marker - RLE_MARKER_BASE);
        if output.len() + run_length >= output_capacity {
            return Err(RleDecodeError::OutputCapacityReached);
        }
        output.extend(std::iter::repeat_n(value, run_length));
        index += 2;
    }

    Ok(output)
}

fn append_rle_run(output: &mut Vec<u8>, byte: u8, mut run_length: usize) {
    if run_length == 1 {
        if byte >= RLE_FIRST_MARKER {
            output.push(RLE_FIRST_MARKER);
        }
        output.push(byte);
        return;
    }

    while run_length > RLE_MAX_RUN {
        output.extend_from_slice(&[u8::MAX, byte]);
        run_length -= RLE_MAX_RUN;
    }
    output.extend_from_slice(&[RLE_MARKER_BASE + run_length as u8, byte]);
}

/// Владеющий буфер общего legacy-сообщения с курсором чтения после заголовка.
pub(crate) struct CBaseMessage {
    data: Vec<u8>,
    cursor: usize,
}

impl CBaseMessage {
    /// Создаёт пустое сообщение с исходным 16-байтовым заголовком и длиной 16.
    pub(crate) fn new() -> Self {
        let mut message = Self {
            data: vec![0; HEADER_LEN],
            cursor: HEADER_LEN,
        };
        message.update_length();
        message
    }

    /// Создаёт сообщение из буквальных шестнадцати байт заголовка и payload.
    ///
    /// Как исходные `CreateMessage*`, нормализует первое слово заголовка до
    /// фактической итоговой длины, сохраняя остальные три слова без толкования.
    pub(crate) fn from_header_and_payload(mut header: [u8; HEADER_LEN], payload: &[u8]) -> Self {
        LegacyMessageHeader::mut_from_bytes(&mut header)
            .expect("массив имеет точный layout legacy header")
            .length
            .set(HEADER_LEN as u32);
        let mut message = Self {
            data: header.to_vec(),
            cursor: HEADER_LEN,
        };
        message.append(payload);
        message
    }

    /// Возвращает всё сообщение в исходном wire-layout, включая заголовок.
    pub(crate) fn as_wire_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Перезаписывает второй Windows `long` заголовка — полный тип сообщения.
    ///
    /// Это точная безопасная форма старых записей `*(message + 4) = opcode`;
    /// payload, курсор чтения и остальные слова заголовка не меняются.
    pub(crate) fn set_message_type(&mut self, message_type: i32) {
        self.header_mut().message_type.set(message_type as u32);
    }

    /// Возвращает второе слово фиксированного header без ручного slicing.
    pub(crate) fn message_type(&self) -> i32 {
        self.header().message_type.get() as i32
    }

    /// Возвращает текущий offset чтения внутри wire-буфера.
    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    /// Заимствует payload от текущего read cursor до конца, не двигая cursor.
    /// Это safe-форма старых relay-вызовов `Add(data + read, size - read)`.
    pub(crate) fn unread_bytes(&self) -> &[u8] {
        self.remaining()
    }

    /// Даёт владельцу составного legacy-типа общий wire-буфер и его cursor.
    ///
    /// Это безопасная форма старой пары `unsigned char* + long&`: владелец
    /// сам последовательно двигает offset, не получая raw pointer к буферу.
    pub(crate) fn wire_bytes_and_cursor_mut(&mut self) -> (&[u8], &mut usize) {
        (&self.data, &mut self.cursor)
    }

    /// Перезаписывает поле длины фактическим размером буфера, как старый
    /// inline `Update`.
    pub(crate) fn update(&mut self) {
        self.update_length();
    }

    /// Читает знаковый однобайтовый `char` или не двигает курсор при нехватке.
    pub(crate) fn get_char(&mut self) -> Option<i8> {
        self.take_array().map(i8::from_le_bytes)
    }

    /// Читает беззнаковый однобайтовый `unsigned char`.
    pub(crate) fn get_byte(&mut self) -> Option<u8> {
        self.take_array().map(u8::from_le_bytes)
    }

    /// Читает знаковый 16-битный `short` в little-endian порядке.
    pub(crate) fn get_short(&mut self) -> Option<i16> {
        self.take_array().map(i16::from_le_bytes)
    }

    /// Читает беззнаковый 16-битный `unsigned short`.
    pub(crate) fn get_word(&mut self) -> Option<u16> {
        self.take_array().map(u16::from_le_bytes)
    }

    /// Читает 32-битный Windows `long`.
    pub(crate) fn get_long(&mut self) -> Option<i32> {
        self.take_array().map(i32::from_le_bytes)
    }

    /// Читает 64-битный `__int64`.
    pub(crate) fn get_long64(&mut self) -> Option<i64> {
        self.take_array().map(i64::from_le_bytes)
    }

    /// Читает 32-битный IEEE-754 `float`.
    pub(crate) fn get_float(&mut self) -> Option<f32> {
        self.take_array().map(f32::from_le_bytes)
    }

    /// Копирует ровно `destination.len()` байт и двигает курсор только при
    /// полном успехе, как исходный сырой `Get`.
    pub(crate) fn get(&mut self, destination: &mut [u8]) -> bool {
        let Some(source) = self.remaining().get(..destination.len()) else {
            return false;
        };
        destination.copy_from_slice(source);
        self.cursor += destination.len();
        true
    }

    /// Читает bytes до NUL по контракту Auth `CMessage::GetStr` RVA
    /// `0x000138C0` и двигает курсор также через завершающий байт.
    ///
    /// Если NUL отсутствует, курсор остаётся в конце сообщения, а исходная
    /// повторная очистка результата выражена пустым `Vec` без частичной строки.
    pub(crate) fn get_c_string_bytes(&mut self) -> Vec<u8> {
        let remaining = self.remaining();
        let Some(terminator) = remaining.iter().position(|byte| *byte == 0) else {
            self.cursor = self.data.len();
            return Vec::new();
        };
        let value = remaining[..terminator].to_vec();
        self.cursor += terminator + 1;
        value
    }

    /// Читает ограниченную C-строку по контракту исходного `GetStr`.
    ///
    /// При NUL до границы возвращает bytes перед ним. Если граница достигнута
    /// без NUL, возвращает пустой результат, но сохраняет уже сдвинутый курсор:
    /// старый код в этом случае занулял первый байт выходного массива.
    /// При нулевой границе exact GameServer `GetStr` всё равно потребляет один
    /// байт. Ненулевой байт оставлял C-строку без терминатора и приводил к UB у
    /// caller-а; Rust сохраняет cursor и безопасно нормализует результат к пустому.
    pub(crate) fn get_str_bytes(&mut self, maximum: usize) -> Option<Vec<u8>> {
        if maximum == 0 {
            let _ = self.get_byte();
            return Some(Vec::new());
        }

        let mut value = Vec::new();
        for _ in 0..maximum {
            let byte = self.get_byte().unwrap_or(0);
            if byte == 0 {
                return Some(value);
            }
            value.push(byte);
        }
        Some(Vec::new())
    }

    /// Читает GUID с исходным однобайтовым маркером присутствия.
    pub(crate) fn get_guid(&mut self) -> Option<CGuid> {
        if self.get_byte()? == 0 {
            return None;
        }

        let mut bytes = [0; 16];
        self.get(&mut bytes)
            .then(|| CGuid::from_legacy_bytes(bytes))
    }

    /// Добавляет знаковый однобайтовый `char`.
    pub(crate) fn add_char(&mut self, value: i8) {
        self.append(&value.to_le_bytes());
    }

    /// Добавляет беззнаковый однобайтовый `unsigned char`.
    pub(crate) fn add_byte(&mut self, value: u8) {
        self.append(&value.to_le_bytes());
    }

    /// Добавляет знаковый 16-битный `short`.
    pub(crate) fn add_short(&mut self, value: i16) {
        self.append(&value.to_le_bytes());
    }

    /// Добавляет беззнаковый 16-битный `unsigned short`.
    pub(crate) fn add_word(&mut self, value: u16) {
        self.append(&value.to_le_bytes());
    }

    /// Добавляет 32-битный Windows `long`.
    pub(crate) fn add_long(&mut self, value: i32) {
        self.append(&value.to_le_bytes());
    }

    /// Добавляет 32-битный Windows `unsigned long`.
    pub(crate) fn add_ulong(&mut self, value: u32) {
        self.append(&value.to_le_bytes());
    }

    /// Добавляет 64-битный `__int64`.
    pub(crate) fn add_long64(&mut self, value: i64) {
        self.append(&value.to_le_bytes());
    }

    /// Добавляет произвольные байты без префикса размера.
    pub(crate) fn add(&mut self, value: &[u8]) {
        self.append(value);
    }

    /// Добавляет nullable C-строку вместе с завершающим нулём.
    pub(crate) fn add_str(&mut self, value: Option<&CStr>) {
        if let Some(value) = value {
            self.append(value.to_bytes_with_nul());
        }
    }

    /// Добавляет 32-битную длину и затем сами байты, как исходный `AddEx`.
    pub(crate) fn add_ex(&mut self, value: &[u8]) {
        self.add_long(value.len() as u32 as i32);
        self.append(value);
    }

    /// Добавляет GUID с маркером `0` для `GUID_INVALID` либо `0x10` и шестнадцатью
    /// legacy-байтами для реального значения.
    pub(crate) fn add_guid(&mut self, value: CGuid) {
        if value.is_invalid() {
            self.add_byte(0);
        } else {
            self.add_byte(16);
            self.append(value.as_legacy_bytes());
        }
    }

    fn remaining(&self) -> &[u8] {
        self.data.get(self.cursor..).unwrap_or_default()
    }

    fn take_array<const N: usize>(&mut self) -> Option<[u8; N]> {
        let value: [u8; N] = self.remaining().get(..N)?.try_into().ok()?;
        self.cursor += N;
        Some(value)
    }

    fn append(&mut self, value: &[u8]) {
        self.data.extend_from_slice(value);
        self.update_length();
    }

    fn update_length(&mut self) {
        // BLOCKED_MISSING_FACT: достижим ли в исходном 32-битном процессе
        // message buffer, чей размер переполняет Windows `long`, и какое
        // наблюдаемое поведение после этого требуется сохранить? Машинная
        // запись оставляла младшие 32 бита; здесь они сохранены, но совместимость
        // последующего чтения буфера больше u32::MAX пока не утверждается.
        let legacy_length = self.data.len() as u32;
        self.header_mut().length.set(legacy_length);
    }

    fn header(&self) -> &LegacyMessageHeader {
        LegacyMessageHeader::ref_from_bytes(&self.data[..HEADER_LEN])
            .expect("CBaseMessage всегда содержит фиксированный legacy header")
    }

    fn header_mut(&mut self) -> &mut LegacyMessageHeader {
        LegacyMessageHeader::mut_from_bytes(&mut self.data[..HEADER_LEN])
            .expect("CBaseMessage всегда содержит фиксированный legacy header")
    }
}

impl Default for CBaseMessage {
    fn default() -> Self {
        Self::new()
    }
}

// Неперенесённый контракт (локальный анализ): `GetStr` (Billing RVA 0x0000CF40, Login 0x00065CA0,
// Misc 0x00010D20, Game 0x00012CF0, World 0x000237F0) читает по байту,
// включая NUL, пока не достигнет `param_2`. При нехватке входа либо отсутствии
// NUL в пределах лимита он записывает NUL в `param_1[0]`, но не откатывает уже
// сдвинутый курсор:
//
// `byte = GetChar(); param_1[i++] = byte;`
// `while (byte != 0 && i < param_2);`
// `if (i == param_2 && param_1[param_2 - 1] != 0) param_1[0] = 0;`
//
// BLOCKED_MISSING_FACT: встречается ли вызов `GetStr` с `param_2 <= 0` и какое
// наблюдаемое поведение требуется сохранить? При нуле оригинал обращается к
// `param_1[-1]`; безопасный Rust не назначает этому пути реакцию по догадке.

// Неперенесённый контракт (локальный анализ): `GetEx` (Login RVA 0x00065D60, Game 0x00012DA0) сначала
// проверяет только `cursor + param_2 <= len`, затем читает 4-байтовую объявленную
// длину и при равенстве копирует ещё `param_2` байт без второй bounds-check:
//
// `if (m_lPtr + param_2 <= size) {`
// `    length = GetLong();`
// `    if (length == param_2) { memcpy(out, data + m_lPtr, param_2); ... }`
// `}`
//
// BLOCKED_MISSING_FACT: достижим ли извне буфер, в котором первая проверка
// проходит, объявленная длина совпадает, но после четырёх байт префикса payload
// короче `param_2`, и как реагирует целевой процесс? До ответа `GetEx` не имеет
// Rust API; корректно сформированный парный `AddEx` уже реализован.
