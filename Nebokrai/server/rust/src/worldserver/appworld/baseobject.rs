//! Владелец базового object-состояния исторического `WorldServer`.
//!
//! Статус PDB-inline `CBaseObject::GetType/SetType`, `GetID/SetID`,
//! `GetExID/SetExID`, `GetGraphicsID/SetGraphicsID`, `GetName`, public
//! `m_bIncludeChild`,
//! `CBaseObject::SetName` RVA `0x0000D320`, достигнутых нулевых scalar-записей,
//! `CBaseObject::AddToByteArray` RVA `0x000D55C0`, вызова `CGUID::CGUID` и
//! `CBaseObject::DecordFromByteArray` RVA `0x000D5820`, пустого имени внутри
//! `CBaseObject::CBaseObject` RVA `0x000D5790` — `IMPLEMENTED`. Контракт
//! child-tree, destructor-а и передачи factory-результата внутри
//! `CreateChildObject` RVA `0x000D58B0` восстановлен по raw/PDB и точечно
//! проверен в EXE, но его Rust-storage ещё не материализован; соответствующие
//! тела ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.h` и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:22`.
//!
//! Точный PDB задаёт размер старого `CBaseObject` `0x50` и три protected
//! signed `long`: `m_lType` по offset `+0x4`, `m_lID` по `+0x8` и
//! `m_lGraphicsID` по `+0x1C`, `CGUID m_guExID` по `+0xC`, а также
//! `std::string m_strName` по `+0x20` и public `bool m_bIncludeChild` по
//! `+0x3C`. Соответствующие public inline-методы читают либо присваивают одно
//! поле. Raw `SetName` копирует только prefix до первого NUL, а constructor
//! ставит include-child в `true`.
//! Соседние protected-поля — non-owning `CBaseObject* m_pFather` по `+0x40` и
//! `std::list<CBaseObject*> m_listObject` по `+0x44`; public inline
//! `GetFather/SetFather` прямо читают и присваивают pointer, а `GetObjectList`
//! возвращает mutable pointer на сам список. Constructor создаёт пустой список
//! и ставит father в `nullptr`.
//! Rust хранит scalar-значения как `i32`, а GUID — готовым общим `CGuid`, чей
//! точный 16-байтовый Microsoft layout, нулевой constructor и четырёхсловное
//! присваивание уже восстановлены отдельно. Имя хранится как owned `Vec<u8>`:
//! кодировка и UTF-8 не навязываются, а внутренний storage и освобождение
//! `std::string` заменены стандартным владением Rust. `GetExID` сохраняет
//! исходное const-заимствование; `SetExID` принимает неизменяемую ссылку,
//! поскольку старому mutable reference была нужна только копия значения.
//! Сразу разыменовываемый `char const*` в `SetName` заменён живым slice, а
//! возвращаемый `GetName` `c_str()` — slice сохранённых байтов без служебного
//! NUL: terminator добавляется только будущей wire/file-границей. Rust layout
//! всего `CBaseObject` не объявляется копией старого ABI/vtable.
//! `AddToByteArray` дописывает в существующий buffer type, ID и graphics ID как
//! три четырёхбайтовых little-endian signed значения, затем byte-exact имя и
//! один NUL. Exact World-overload-ы `_AddToByteArray` RVA
//! `0x000A3340/0x000A3280` подтверждают ширину, порядок in-memory x86 bytes и
//! terminator. `Vec::extend_from_slice/push` заменяют их STL-вставки; входной
//! `bool` исходное тело не читает, а результат всегда равен `true`.
//! Обратный decoder читает те же поля в том же порядке и двигает caller-owned
//! cursor на `12 + name.len() + 1`. World `_GetStringFromByteArray` RVA
//! `0x000A3190` подтверждает включение NUL в cursor. Raw decompiler смешал
//! возвращаемый `bool` с `__security_check_cookie`; точечный разбор exact EXE
//! подтвердил `mov al, 1` по `0x004D589D` перед cookie-check, поэтому normal
//! return равен `true` (`VERIFIED_DISASSEMBLY`). Входной `bool` снова не
//! читается.
//! Старые pointer/`long&` заменены `&[u8]` и `&mut usize`. Безразмерный helper
//! мог читать за источником, а локальный `char[256]` — переполняться; safe Rust
//! возвращает typed `BLOCKED_MISSING_FACT` только на этих границах, сохраняет
//! уже выполненные scalar-присваивания и cursor, но не назначает старому UB
//! fail-closed результат. Имя присваивается лишь после найденного NUL, как после
//! завершения исходного временного buffer.
//! Исторически public include-child хранится отдельным `pub(crate) bool` и
//! получает доказанный default `true`; accessor не придуман, а оба wire-метода
//! поле не читают. Constructor-helper материализует только шесть достигнутых
//! полей; father и child-list остаются в полном raw-конструкторе.
//!
//! Raw child-tree задаёт ownership буквально. `AddObject` сначала записывает
//! father, затем добавляет pointer в хвост без duplicate/null проверки.
//! `RemoveObject` выполняет `std::list::remove`, то есть убирает все равные
//! pointers, но не очищает father и не уничтожает объект. Pointer-вариант
//! `DeleteChildObject` сначала virtual-вызывает `RemoveObject`, затем при
//! ненулевом child вызывает его scalar deleting destructor. `DeleteAllChildObject`
//! копирует список, проходит snapshot в исходном порядке и через virtual
//! `DeleteChildObject` удаляет всё, кроме явно переданного исключения; destructor
//! передаёт `nullptr`, поэтому parent рекурсивно владеет всеми оставшимися
//! детьми. Exact PDB не содержит override-объявлений этих методов у других
//! классов World-корпуса.
//!
//! `CBaseObject::~CBaseObject` RVA `0x000D5D80` сначала ставит base-vtable,
//! удаляет всех детей, освобождает list/string storage и завершает GUID-cleanup.
//! Сырой хвост ошибочно подписал последний вызов как `AddPlayerList(unaff_ESI)`:
//! exact EXE показывает вызов `0x00401000` с `this+0xC`, а по этому адресу
//! находится единственный `ret`. Внешнего player-callback нет
//! (`VERIFIED_DISASSEMBLY`, `0x004D5D80..0x004D5DFD`).
//!
//! `CreateChildObject` сначала получает объект из `CreateObject`, который уже
//! назначил входные type/ID, и до решения об attachment копирует ненулевое имя.
//! Сохранённый затем ID поэтому равен factory-ID. Player type `400` с ID `0`
//! всегда пропускает `AddObject`; `CGoods` type `700` с ID `0` пропускает его
//! только при точном сравнении шести bytes `NoAdd\0`. Во всех остальных
//! случаях virtual `AddObject` parent-а ставит father и добавляет child в хвост
//! до первого virtual-вызова самого child. Detached-ветви всё равно продолжают
//! оставшуюся инициализацию и возвращают живой объект с нулевым father.
//!
//! Первый child-slot `+0x8` является `Load`. Exact EXE-vtable у `CRegion`
//! `0x00548DEC` содержит по этому slot `0x004D7570`, то есть именованный
//! `CRegion::Load`; vtable `CPlayer/CNpc/CMonster/CGoods` соответственно
//! `0x00543DE4/0x00549ED4/0x00549E2C/0x0054296C` содержит общий адрес
//! `0x004DD3E0`, чьё тело возвращает `1`. Raw ошибочно приписал этот общий
//! COMDAT `std::codecvt_base::do_max_length`, но topology slot-а и Region
//! override определяют исходный virtual-контракт. Результат `Load` не
//! проверяется. Перед вызовом восстанавливается сохранённый factory-ID, после
//! него ID безусловно снова назначается входным значением: возможная мутация
//! ID внутри `Load` отбрасывается. Входное имя доступно `Load` благодаря
//! ранней копии, а ненулевой graphics ID назначается только после вызова;
//! поздняя повторная копия имени отбрасывает его возможное изменение.
//! `BaseObjectFactoryType` материализует только подтверждённое сопоставление
//! пяти literal type и concrete factory class. Он не подменяет сам factory:
//! разнородные `CRegion/CPlayer/CNpc/CMonster/CGoods` пока имеют отдельные
//! Rust owner-ы и не могут безопасно стать `Box<CBaseObject>` без потери их
//! virtual `Load` и derived-state.
//!
//! Для `CGoods(type=700, id=0, name=nullptr)` raw переходит к шестибайтовому
//! сравнению без null-проверки. Достижимость и наблюдаемая реакция этого
//! старого null-dereference не доказаны (`BLOCKED_MISSING_FACT`); safe Rust не
//! получает придуманного detached/attached либо fail-closed исхода.
//!
//! Точечный xref-аудит exact EXE не нашёл project-call-site
//! `CreateChildObject`. В executable-секциях нет direct `call` к
//! `0x004D58B0`; адрес функции встречается только в 17 vtable-ячейках. Все 27
//! косвенных candidate-инструкций `call [register + 0x28]` сопоставлены с raw и
//! PDB: их receivers принадлежат CrashRpt/CRT, container traversal,
//! `CTimer`, `COrganizing` либо ADO/DB wrappers, но не `CBaseObject`. В
//! частности, единственный четырёхаргументный project-candidate по `0x004A73CD`
//! принадлежит `COrganizing` в `OnOrgasysMessage`; его собственная vtable
//! начинается по `0x00549984`, а не в одной из base-object таблиц.
//! Аналогично точные адреса `RemoveObject` и обеих перегрузок
//! `DeleteChildObject` не имеют внешних direct-call, а найденные одноимённые
//! slot-offset-ы принадлежат другим vtable. Внутри самого owner-а остаётся
//! только уже классифицированная цепочка: `CreateChildObject` virtual-вызывает
//! `AddObject`, pointer-delete — `RemoveObject`, key-delete и
//! `DeleteAllChildObject` — pointer-delete, destructor прямо вызывает
//! `DeleteAllChildObject`. Поиск полей `m_listObject/m_pFather` и трёх PDB-
//! inline accessor-ов в raw/PDB-корпусе также не дал использования вне этого
//! файла.
//!
//! Следовательно, exact World-проект не сохраняет возвращённый pointer,
//! detached player/goods и alias после `RemoveObject`: он вообще не достигает
//! публичной child-tree границы. Это доказательство отсутствия текущего
//! project-caller-а, а не доказательство желаемого Rust-владения и не
//! разрешение удалить публичный исходный контракт.
//!
//! Rust-storage намеренно не выбран заранее. Список содержит гетерогенные
//! объекты пяти factory-типов, удаление инвалидирует все raw aliases немедленно,
//! а отдельный `RemoveObject` формально оставляет живой detached pointer со
//! старым father. До материализации `CreateObject` и derived owner-ов нельзя
//! доказать совместимость `Box`, `Rc`, `Arc`, `Pin` либо raw-pointer слоя.
//! Заменённые
//! raw-блоки `SetName`, `AddToByteArray` и `DecordFromByteArray` удалены;
//! отдельных экспортированных тел остальных inline-методов в корпусе нет, а
//! их контракт полностью определён PDB.

use std::error::Error;
use std::fmt;

use crate::public::guid::CGuid;

const LEGACY_NAME_CAPACITY: usize = 0x100;

/// Ошибка безопасной границы старого byte-array decoder-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseObjectDecodeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    LegacyNameOverflow {
        first_out_of_bounds_offset: usize,
    },
}

impl fmt::Display for BaseObjectDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
            Self::LegacyNameOverflow {
                first_out_of_bounds_offset,
            } => write!(
                formatter,
                "имя вышло за старый 256-байтовый буфер на offset {first_out_of_bounds_offset}"
            ),
        }
    }
}

impl Error for BaseObjectDecodeError {}

/// Ровно пять literal type, принимаемых `CBaseObject::CreateObject`.
///
/// Это discriminator factory, а не полный wire-enum: прочие signed type могут
/// существовать в сохранённых объектах, но exact factory возвращает для них
/// null. Значения подтверждены ветвями `0x004D5470..0x004D55B3`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseObjectFactoryType {
    Region,
    Player,
    Npc,
    Monster,
    Goods,
}

impl BaseObjectFactoryType {
    /// Возвращает concrete owner только для пяти factory-ветвей original-а.
    pub(crate) const fn from_wire_value(value: i32) -> Option<Self> {
        match value {
            200 => Some(Self::Region),
            400 => Some(Self::Player),
            500 => Some(Self::Npc),
            600 => Some(Self::Monster),
            700 => Some(Self::Goods),
            _ => None,
        }
    }

    /// Возвращает исходный signed `long`, записываемый после factory constructor-а.
    pub(crate) const fn wire_value(self) -> i32 {
        match self {
            Self::Region => 200,
            Self::Player => 400,
            Self::Npc => 500,
            Self::Monster => 600,
            Self::Goods => 700,
        }
    }
}

/// Достигнутая часть исходного `CBaseObject`.
pub(crate) struct CBaseObject {
    object_type: i32,
    id: i32,
    ex_id: CGuid,
    graphics_id: i32,
    name: Vec<u8>,
    /// Сохраняет public-флаг включения дочерних объектов без wire-эффекта.
    pub(crate) include_child: bool,
}

impl CBaseObject {
    /// Создаёт только уже достигнутые начальные состояния конструктора.
    pub(crate) const fn with_reached_constructor_defaults() -> Self {
        Self {
            object_type: 0,
            id: 0,
            ex_id: CGuid::GUID_INVALID,
            graphics_id: 0,
            name: Vec::new(),
            include_child: true,
        }
    }

    /// Возвращает signed Windows `long` type без преобразования битов.
    pub(crate) const fn get_type(&self) -> i32 {
        self.object_type
    }

    /// Идентифицирует factory-вариант, не превращая неизвестный type в default.
    pub(crate) const fn factory_type(&self) -> Option<BaseObjectFactoryType> {
        BaseObjectFactoryType::from_wire_value(self.object_type)
    }

    /// Присваивает signed Windows `long` type без дополнительных эффектов.
    pub(crate) const fn set_type(&mut self, object_type: i32) {
        self.object_type = object_type;
    }

    /// Возвращает signed Windows `long` ID без преобразования битов.
    pub(crate) const fn get_id(&self) -> i32 {
        self.id
    }

    /// Присваивает signed Windows `long` ID без дополнительных эффектов.
    pub(crate) const fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    /// Заимствует GUID в исходной форме `CGUID const&`.
    pub(crate) const fn get_ex_id(&self) -> &CGuid {
        &self.ex_id
    }

    /// Копирует все 16 legacy-байтов GUID без дополнительных эффектов.
    pub(crate) const fn set_ex_id(&mut self, ex_id: &CGuid) {
        self.ex_id = *ex_id;
    }

    /// Возвращает signed Windows `long` graphics ID без преобразования битов.
    pub(crate) const fn get_graphics_id(&self) -> i32 {
        self.graphics_id
    }

    /// Присваивает signed Windows `long` graphics ID без дополнительных эффектов.
    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.graphics_id = graphics_id;
    }

    /// Заимствует сохранённые байты имени без служебного C-терминатора.
    pub(crate) fn get_name(&self) -> &[u8] {
        &self.name
    }

    /// Копирует байты имени только до первого NUL.
    pub(crate) fn set_name(&mut self, name: &[u8]) {
        let prefix_len = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        self.name.clear();
        self.name.extend_from_slice(&name[..prefix_len]);
    }

    /// Дописывает базовые поля в legacy byte-array и всегда сообщает успех.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        _include_child: bool,
    ) -> bool {
        destination.extend_from_slice(&self.object_type.to_le_bytes());
        destination.extend_from_slice(&self.id.to_le_bytes());
        destination.extend_from_slice(&self.graphics_id.to_le_bytes());
        destination.extend_from_slice(&self.name);
        destination.push(0);
        true
    }

    /// Читает базовые поля из legacy byte-array и сохраняет пройденный cursor.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
    ) -> Result<bool, BaseObjectDecodeError> {
        self.object_type = read_legacy_i32(source, cursor, "m_lType")?;
        self.id = read_legacy_i32(source, cursor, "m_lID")?;
        self.graphics_id = read_legacy_i32(source, cursor, "m_lGraphicsID")?;
        self.name = read_legacy_name(source, cursor)?;
        Ok(true)
    }
}

fn read_legacy_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, BaseObjectDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(4) else {
        return Err(BaseObjectDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        // BLOCKED_MISSING_FACT: оригинал сначала сдвигал `long&`, затем читал
        // безразмерный pointer. Результат за концом источника неизвестен;
        // безопасный Rust не проходит через несуществующие байты.
        return Err(BaseObjectDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        });
    };
    *cursor = end;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("slice содержит ровно четыре байта"),
    ))
}

fn read_legacy_name(source: &[u8], cursor: &mut usize) -> Result<Vec<u8>, BaseObjectDecodeError> {
    let mut name = Vec::new();
    loop {
        let offset = *cursor;
        let Some(byte) = source.get(offset).copied() else {
            // BLOCKED_MISSING_FACT: World helper RVA `0x000A3190` не знал
            // длину источника и продолжал чтение до NUL. Реакция при
            // отсутствующем terminator неизвестна, поэтому байт и cursor не
            // придумываются.
            return Err(BaseObjectDecodeError::UnexpectedEnd {
                field: "m_strName",
                offset,
                needed: 1,
                available: 0,
            });
        };
        *cursor = offset + 1;

        if name.len() == LEGACY_NAME_CAPACITY {
            // BLOCKED_MISSING_FACT: helper уже потребил этот байт перед
            // записью за local `char[256]`; достижимость и результат такого
            // повреждения stack не доказаны.
            return Err(BaseObjectDecodeError::LegacyNameOverflow {
                first_out_of_bounds_offset: offset,
            });
        }
        if byte == 0 {
            return Ok(name);
        }
        name.push(byte);
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp

// IMPLEMENTED: `CBaseObject::SetName` RVA `0x0000D320` находится выше;
// owned bytes сохраняют точный C-string prefix без STL storage.


// ============================================================================
// FUNCTION: CBaseObject::DeleteChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:280
// RVA: 0x000D5420
// ADDRESS: 004d5420
// PROTOTYPE: void __thiscall DeleteChildObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::DeleteChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:291
// RVA: 0x000D5440
// ADDRESS: 004d5440
// PROTOTYPE: void __thiscall DeleteChildObject(long param_1, long param_2, CGUID * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::CreateObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:157
// RVA: 0x000D5470
// ADDRESS: 004d5470
// PROTOTYPE: CBaseObject * __cdecl CreateObject(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CBaseObject::AddToByteArray` RVA `0x000D55C0` находится выше;
// `Vec` сохраняет append-порядок и wire bytes без STL storage.

// ============================================================================
// FUNCTION: CBaseObject::FindChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:84
// RVA: 0x000D5620
// ADDRESS: 004d5620
// PROTOTYPE: bool __thiscall FindChildObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::FindChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:100
// RVA: 0x000D5650
// ADDRESS: 004d5650
// PROTOTYPE: CBaseObject * __thiscall FindChildObject(long param_1, long param_2, CGUID * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::RecursiveFindObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:115
// RVA: 0x000D5680
// ADDRESS: 004d5680
// PROTOTYPE: CBaseObject * __thiscall RecursiveFindObject(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::RecursiveFindObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:136
// RVA: 0x000D56D0
// ADDRESS: 004d56d0
// PROTOTYPE: CBaseObject * __thiscall RecursiveFindObject(long param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::BoardCast
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:400
// RVA: 0x000D5750
// ADDRESS: 004d5750
// PROTOTYPE: void __thiscall BoardCast(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::RemoveObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:321
// RVA: 0x000D5780
// ADDRESS: 004d5780
// PROTOTYPE: void __thiscall RemoveObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::CBaseObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:22
// RVA: 0x000D5790
// ADDRESS: 004d5790
// PROTOTYPE: undefined __thiscall CBaseObject(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// VERIFIED_DISASSEMBLY: `CBaseObject::DecordFromByteArray` RVA `0x000D5820`
// находится выше; normal return `true` подтверждён `mov al, 1` по `0x004D589D`.

// VERIFIED_DISASSEMBLY: `CBaseObject::~CBaseObject` RVA `0x000D5D80` ниже не
// вызывает наблюдаемый `AddPlayerList`: call по `0x004D5DE9` приходит в один
// `ret` по `0x00401000`. Остальное тело сохраняется raw до materialization
// child/factory ownership.

// CLASSIFIED_RAW: `CreateChildObject` RVA `0x000D58B0` передаёт child в
// `AddObject` до virtual `Load`, кроме player ID 0 и точного goods
// `(ID 0, "NoAdd\0")`; результат `Load` игнорируется, а ID/graphics/name затем
// назначаются в доказанном порядке. Storage и handle намеренно не выбраны.
// BLOCKED_MISSING_FACT: достижим ли `CGoods(type=700, id=0, name=nullptr)` и
// как exact процесс наблюдаемо завершает его безусловное сравнение name?

// VERIFIED_DISASSEMBLY: в exact World EXE нет project-call-site
// `CreateChildObject` RVA `0x000D58B0`; 17 ссылок являются vtable-ячейками, а
// все 27 candidate-инструкций `call [register + 0x28]` принадлежат другим
// классам. Поэтому
// возвращённый child и detached-ветви не создают project-alias, но handle из
// одного только отсутствия caller-а не выбирается.

// ============================================================================
// FUNCTION: CBaseObject::CreateChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:204
// RVA: 0x000D58B0
// ADDRESS: 004d58b0
// PROTOTYPE: CBaseObject * __thiscall CreateChildObject(long param_1, long param_2, char * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::AddObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:340
// RVA: 0x000D5AD0
// ADDRESS: 004d5ad0
// PROTOTYPE: void __thiscall AddObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::DeleteAllChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:303
// RVA: 0x000D5B90
// ADDRESS: 004d5b90
// PROTOTYPE: void __thiscall DeleteAllChildObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::DgFindObjectsByTypes
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:360
// RVA: 0x000D5C50
// ADDRESS: 004d5c50
// PROTOTYPE: void __thiscall DgFindObjectsByTypes(long param_1, _func_long_long_long * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:381
// RVA: 0x000D5CF0
// ADDRESS: 004d5cf0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::~CBaseObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:33
// RVA: 0x000D5D80
// ADDRESS: 004d5d80
// PROTOTYPE: void __thiscall ~CBaseObject(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//












// COMPONENT_VARIANT_END: WorldServer
