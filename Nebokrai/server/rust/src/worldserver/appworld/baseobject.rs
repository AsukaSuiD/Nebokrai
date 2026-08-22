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
//! проверен в EXE. `CreateChildObject`, `AddObject`, `RemoveObject`, обе
//! перегрузки `FindChildObject`, обе `RecursiveFindObject` и обе
//! `DeleteChildObject`, `DeleteAllChildObject`, `BoardCast`,
//! `DgFindObjectsByTypes`, `AI`, constructor и destructor materialизованы
//! безопасным child-tree owner-ом. Статический `CreateObject`
//! RVA `0x000D5470` materialизован отдельным tagged factory-result без erased
//! pointer/vtable. Точная пара:
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
//! `Drop` у tree-node сначала освобождает parent ownership children в list-order,
//! затем Rust освобождает base object. Это сохраняет наблюдаемый lifecycle без
//! base-vtable/CRT cleanup и без dangling child aliases original-а.
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
//! `BaseObjectFactoryType` и `BaseObjectFactoryObject` materialизуют
//! подтверждённое сопоставление пяти literal type и concrete factory class.
//! Tagged `Box` сохраняет heap-owner результата без erased `CBaseObject*`:
//! разнородные `CRegion/CPlayer/CNpc/CMonster/CGoods` остаются собственными
//! Rust owner-ами и не теряют derived-state либо virtual `Load`.
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
//! Rust child-tree хранится безопасным `BaseObjectTreeNode`: гетерогенный
//! factory-owner остаётся tagged, parent владеет children в list-order, а
//! child держит только `Weak` father. `CreateChildObject` возвращает
//! `Rc<RefCell<...>>`, а точное действие virtual `AddObject` передаёт
//! вызывающему owner-у explicit callback-ом до `Load`. `AddObject` сохраняет
//! duplicate insertion и overwrite father; только цикл, который в original-е
//! приводил бы к неограниченной рекурсии и dangling lifetime, safe Rust
//! отвергает до мутации. `RemoveObject` убирает все equal children и, как EXE,
//! не очищает father. Такое Rust-владение исключает erased pointer и старую
//! утечку на null-ветви, не приписывая отсутствующему caller-у container-
//! semantics. Для `CGoods(id=0, name=nullptr)` старый EXE
//! разыменовывал null: безопасная граница возвращает typed error без
//! attachment и `Load`. Короткое имя больше не читается за границей slice и
//! просто не совпадает с `NoAdd\0`: это исправление внутреннего UB без
//! доказанного внешнего legacy-эффекта.
//! Заменённые
//! raw-блоки `SetName`, `AddToByteArray` и `DecordFromByteArray` удалены;
//! отдельных экспортированных тел остальных inline-методов в корпусе нет, а
//! их контракт полностью определён PDB.

use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

use super::goods::cgoods::CGoods;
use super::monster::CMonster;
use super::npc::CNpc;
use super::player::CPlayer;
use super::region::CRegion;
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

/// Безопасная замена единственной достигаемой null-границы
/// `CBaseObject::CreateChildObject`.
///
/// Только `CGoods(type=700, factory-ID=0)` обращался к `name` без проверки на
/// null ради сравнения `NoAdd\0`. До этой точки object уже построен и имя
/// могло быть присвоено, однако exact EXE немедленно аварийно завершался до
/// `AddObject` и `Load`. Rust не сохраняет этот внутренний дефект: allocation
/// освобождается обычным владением, а вызывающий получает явный результат.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseObjectChildCreateError {
    MissingGoodsNameForNoAddProbe,
}

impl fmt::Display for BaseObjectChildCreateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGoodsNameForNoAddProbe => write!(
                formatter,
                "товар с нулевым factory-ID требует имя для проверки NoAdd"
            ),
        }
    }
}

impl Error for BaseObjectChildCreateError {}

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

/// Safe tagged owner результата `CBaseObject::CreateObject`.
///
/// Старый factory возвращал base-pointer на пять разнородных heap-объектов.
/// `Box` сохраняет единое владение результатом, а enum заменяет только erased
/// pointer/vtable: concrete object и его последующий `Load` не теряются.
pub(crate) enum BaseObjectFactoryObject {
    Region(CRegion),
    Player(CPlayer),
    Npc(CNpc),
    Monster(CMonster),
    Goods(CGoods),
}

/// Безопасная причина отклонения internal cycle старого child-tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseObjectTreeAttachError {
    WouldCreateCycle,
}

impl fmt::Display for BaseObjectTreeAttachError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WouldCreateCycle => write!(formatter, "добавление child создаёт цикл object-tree"),
        }
    }
}

impl Error for BaseObjectTreeAttachError {}

/// Безопасный узел historic `m_pFather + m_listObject`.
///
/// `Vec` сохраняет list-order `AddObject`; его strong ownership заменяет
/// deleting destructor parent-а. `father` намеренно weak: raw pointer не
/// владел parent-ом. Duplicate children допустимы как в original-е, а cycle
/// отвергается до изменения state, поскольку его рекурсивные owner-ы не имели
/// доказанного внешнего эффекта, но приводили к internal lifetime defect.
pub(crate) struct BaseObjectTreeNode {
    object: BaseObjectFactoryObject,
    father: Option<std::rc::Weak<RefCell<BaseObjectTreeNode>>>,
    children: Vec<SharedBaseObjectFactoryObject>,
}

/// Живой heterogeneous child без erased raw-pointer старого ABI.
pub(crate) type SharedBaseObjectFactoryObject = Rc<RefCell<BaseObjectTreeNode>>;

impl BaseObjectTreeNode {
    fn from_object(object: BaseObjectFactoryObject) -> Self {
        Self {
            object,
            father: None,
            children: Vec::new(),
        }
    }

    /// Создаёт root узел через точные пять ветвей `CreateObject`.
    pub(crate) fn create(
        object_type: i32,
        object_id: i32,
    ) -> Option<SharedBaseObjectFactoryObject> {
        create_base_object(object_type, object_id)
            .map(|object| Rc::new(RefCell::new(Self::from_object(*object))))
    }

    /// Заимствует tagged concrete owner без erased virtual ABI.
    pub(crate) fn object(&self) -> &BaseObjectFactoryObject {
        &self.object
    }

    /// Заимствует concrete owner для точного `Load`/post-load lifecycle.
    pub(crate) fn object_mut(&mut self) -> &mut BaseObjectFactoryObject {
        &mut self.object
    }

    /// Возвращает текущего father либо `None` для root/detached node.
    pub(crate) fn father(&self) -> Option<SharedBaseObjectFactoryObject> {
        self.father.as_ref().and_then(std::rc::Weak::upgrade)
    }

    /// Возвращает snapshot children в exact insertion-order.
    pub(crate) fn children(&self) -> Vec<SharedBaseObjectFactoryObject> {
        self.children.clone()
    }

    /// Выполняет `CBaseObject::AddObject`: father, затем append child.
    pub(crate) fn add_child(
        parent: &SharedBaseObjectFactoryObject,
        child: &SharedBaseObjectFactoryObject,
    ) -> Result<(), BaseObjectTreeAttachError> {
        if Self::contains_node(child, parent) {
            return Err(BaseObjectTreeAttachError::WouldCreateCycle);
        }
        child.borrow_mut().father = Some(Rc::downgrade(parent));
        parent.borrow_mut().children.push(child.clone());
        Ok(())
    }

    /// Выполняет `CBaseObject::RemoveObject`: удаляет все pointer-equal children
    /// и, как EXE, оставляет child.father неизменённым.
    pub(crate) fn remove_child(
        parent: &SharedBaseObjectFactoryObject,
        child: &SharedBaseObjectFactoryObject,
    ) {
        parent
            .borrow_mut()
            .children
            .retain(|candidate| !Rc::ptr_eq(candidate, child));
    }

    /// Повторяет pointer-identity `FindChildObject(CBaseObject*)`.
    pub(crate) fn find_child(
        parent: &SharedBaseObjectFactoryObject,
        child: &SharedBaseObjectFactoryObject,
    ) -> bool {
        parent
            .borrow()
            .children
            .iter()
            .any(|candidate| Rc::ptr_eq(candidate, child))
    }

    /// Повторяет list-order `FindChildObject(type, id, ignored_guid)`.
    pub(crate) fn find_child_by_type_and_id(
        parent: &SharedBaseObjectFactoryObject,
        object_type: i32,
        object_id: i32,
    ) -> Option<SharedBaseObjectFactoryObject> {
        parent
            .borrow()
            .children
            .iter()
            .find(|candidate| {
                let candidate = candidate.borrow();
                candidate.object.object_type() == object_type
                    && candidate.object.object_id() == object_id
            })
            .cloned()
    }

    /// Повторяет preorder `RecursiveFindObject(type, id)`.
    pub(crate) fn recursive_find_by_type_and_id(
        root: &SharedBaseObjectFactoryObject,
        object_type: i32,
        object_id: i32,
    ) -> Option<SharedBaseObjectFactoryObject> {
        Self::recursive_find(root, &mut |object| {
            object.object_type() == object_type && object.object_id() == object_id
        })
    }

    /// Повторяет preorder `RecursiveFindObject(type, c_string_name)`.
    pub(crate) fn recursive_find_by_type_and_name(
        root: &SharedBaseObjectFactoryObject,
        object_type: i32,
        name: &[u8],
    ) -> Option<SharedBaseObjectFactoryObject> {
        let name = legacy_c_string_prefix(name);
        Self::recursive_find(root, &mut |object| {
            object.object_type() == object_type && object.object_name() == name
        })
    }

    /// Выполняет default `DeleteChildObject(child)`: сначала remove всех
    /// совпадений, затем отпускает parent ownership. Существующий внешний
    /// `Rc` сохраняет безопасный живой alias вместо старого dangling pointer.
    pub(crate) fn delete_child(
        parent: &SharedBaseObjectFactoryObject,
        child: &SharedBaseObjectFactoryObject,
    ) {
        Self::remove_child(parent, child);
    }

    /// Выполняет `DeleteChildObject(type, id, ignored_guid)` через первый
    /// list-order hit; отсутствие child остаётся no-op.
    pub(crate) fn delete_child_by_type_and_id(
        parent: &SharedBaseObjectFactoryObject,
        object_type: i32,
        object_id: i32,
    ) {
        if let Some(child) = Self::find_child_by_type_and_id(parent, object_type, object_id) {
            Self::delete_child(parent, &child);
        }
    }

    /// Выполняет `DeleteAllChildObject(exception)` по snapshot исходного list.
    ///
    /// Каждый неравный exception child проходит default delete в исходном
    /// порядке. Snapshot не удерживает удалённое дерево после возврата, а
    /// возможные внешние safe aliases не превращаются в dangling pointers.
    pub(crate) fn delete_all_children_except(
        parent: &SharedBaseObjectFactoryObject,
        exception: Option<&SharedBaseObjectFactoryObject>,
    ) {
        let children = parent.borrow().children.clone();
        for child in children {
            if exception.is_some_and(|exception| Rc::ptr_eq(&child, exception)) {
                continue;
            }
            Self::delete_child(parent, &child);
        }
    }

    /// Выполняет virtual `BoardCast` каждого direct child в list-order.
    ///
    /// Raw проходит сам `std::list`, не создавая snapshot. Вызванный virtual
    /// owner мог удалить текущий list-node и тем самым оставить старый итератор
    /// dangling; внешняя семантика такого invalidated-итератора не определена.
    /// Safe owner берёт snapshot до первого callback: все children, существующие
    /// на входе, получают broadcast в том же порядке, а reentrant mutation не
    /// создаёт memory defect.
    pub(crate) fn broadcast_children(
        parent: &SharedBaseObjectFactoryObject,
        first: i32,
        second: i32,
        broadcast: &mut impl FnMut(&SharedBaseObjectFactoryObject, i32, i32),
    ) {
        let children = parent.borrow().children.clone();
        for child in children {
            broadcast(&child, first, second);
        }
    }

    /// Выполняет `DgFindObjectsByTypes` с точным snapshot direct children.
    ///
    /// Совпавший direct child передаётся callback-у вместе с неизменным
    /// `parameter`; несовпавший рекурсивно ищется в своём поддереве. Как raw,
    /// родительская связь child здесь не участвует. Дерево этого factory-owner-а
    /// содержит только базовые пять concrete типов `CreateObject`, поэтому
    /// рекурсивный base owner соответствует их подтверждённому dispatch.
    pub(crate) fn dg_find_objects_by_type(
        parent: &SharedBaseObjectFactoryObject,
        object_type: i32,
        parameter: i32,
        on_found: &mut impl FnMut(&SharedBaseObjectFactoryObject, i32),
    ) {
        let children = parent.borrow().children.clone();
        for child in children {
            if child.borrow().object.object_type() == object_type {
                on_found(&child, parameter);
            } else {
                Self::dg_find_objects_by_type(&child, object_type, parameter, on_found);
            }
        }
    }

    /// Выполняет virtual `AI` каждого direct child по snapshot исходного list.
    ///
    /// Пустой list остаётся no-op. Callback заменяет virtual slot child-а, а
    /// snapshot сохраняет raw порядок и разрешает callback-у безопасно менять
    /// дерево, не затрагивая текущий проход.
    pub(crate) fn run_child_ai(
        parent: &SharedBaseObjectFactoryObject,
        run_ai: &mut impl FnMut(&SharedBaseObjectFactoryObject),
    ) {
        let children = parent.borrow().children.clone();
        for child in children {
            run_ai(&child);
        }
    }

    fn contains_node(
        root: &SharedBaseObjectFactoryObject,
        sought: &SharedBaseObjectFactoryObject,
    ) -> bool {
        let mut pending = vec![root.clone()];
        let mut seen = HashSet::new();
        while let Some(node) = pending.pop() {
            let address = Rc::as_ptr(&node) as usize;
            if !seen.insert(address) {
                continue;
            }
            if Rc::ptr_eq(&node, sought) {
                return true;
            }
            pending.extend(node.borrow().children.iter().rev().cloned());
        }
        false
    }

    fn recursive_find(
        root: &SharedBaseObjectFactoryObject,
        predicate: &mut impl FnMut(&BaseObjectFactoryObject) -> bool,
    ) -> Option<SharedBaseObjectFactoryObject> {
        if predicate(&root.borrow().object) {
            return Some(root.clone());
        }
        let children = root.borrow().children.clone();
        for child in children {
            if let Some(found) = Self::recursive_find(&child, predicate) {
                return Some(found);
            }
        }
        None
    }
}

impl Drop for BaseObjectTreeNode {
    fn drop(&mut self) {
        // `CBaseObject::~CBaseObject` сперва удаляет children через
        // `DeleteAllChildObject(nullptr)`, затем освобождает base storage.
        // Клон list здесь играет его не-владеющий snapshot: до drop каждого
        // child все его equal entries убираются из parent-list. Повтор одного
        // raw pointer в original-е приводил бы к dangling delete; `Rc` оставляет
        // это внутреннее повреждение безопасным и освобождает alias один раз.
        let children = self.children.clone();
        for child in children {
            self.children
                .retain(|candidate| !Rc::ptr_eq(candidate, &child));
        }
    }
}

impl BaseObjectFactoryObject {
    /// Возвращает исходный type concrete factory-ветви.
    pub(crate) const fn object_type(&self) -> i32 {
        match self {
            Self::Region(object) => object.get_type(),
            Self::Player(object) => object.get_type(),
            Self::Npc(object) => object.get_type(),
            Self::Monster(object) => object.get_type(),
            Self::Goods(object) => object.get_type(),
        }
    }

    /// Возвращает назначенный static factory signed object ID.
    pub(crate) const fn object_id(&self) -> i32 {
        match self {
            Self::Region(object) => object.get_id(),
            Self::Player(object) => object.get_id(),
            Self::Npc(object) => object.get_id(),
            Self::Monster(object) => object.get_id(),
            Self::Goods(object) => object.get_id(),
        }
    }

    fn set_object_id(&mut self, id: i32) {
        match self {
            Self::Region(object) => object.set_id(id),
            Self::Player(object) => object.set_id(id),
            Self::Npc(object) => object.set_id(id),
            Self::Monster(object) => object.set_id(id),
            Self::Goods(object) => object.set_id(id),
        }
    }

    fn set_object_name(&mut self, name: &[u8]) {
        match self {
            Self::Region(object) => object.set_name(name),
            Self::Player(object) => object.set_name(name),
            Self::Npc(object) => object.set_name(name),
            Self::Monster(object) => object.set_name(name),
            Self::Goods(object) => object.set_name(name),
        }
    }

    fn object_name(&self) -> &[u8] {
        match self {
            Self::Region(object) => object.get_name(),
            Self::Player(object) => object.get_name(),
            Self::Npc(object) => object.get_name(),
            Self::Monster(object) => object.get_name(),
            Self::Goods(object) => object.get_goods_name(),
        }
    }

    fn set_object_graphics_id(&mut self, graphics_id: i32) {
        match self {
            Self::Region(object) => object.set_graphics_id(graphics_id),
            Self::Player(object) => object.set_graphics_id(graphics_id),
            Self::Npc(object) => object.set_graphics_id(graphics_id),
            Self::Monster(object) => object.set_graphics_id(graphics_id),
            Self::Goods(object) => object.set_graphics_id(graphics_id),
        }
    }
}

/// Материализует точные пять ветвей `CBaseObject::CreateObject`.
///
/// Каждый concrete constructor уже ставит свой literal type. Как и EXE, ID
/// назначается только после construction; финальная запись того же type не
/// меняет наблюдаемого состояния и не требует искусственного virtual ABI.
pub(crate) fn create_base_object(
    object_type: i32,
    object_id: i32,
) -> Option<Box<BaseObjectFactoryObject>> {
    let mut object = match BaseObjectFactoryType::from_wire_value(object_type)? {
        BaseObjectFactoryType::Region => {
            BaseObjectFactoryObject::Region(CRegion::with_constructor_base_and_type())
        }
        BaseObjectFactoryType::Player => {
            BaseObjectFactoryObject::Player(CPlayer::with_clone_decode_constructor_state())
        }
        BaseObjectFactoryType::Npc => {
            BaseObjectFactoryObject::Npc(CNpc::with_constructor_base_and_type())
        }
        BaseObjectFactoryType::Monster => {
            BaseObjectFactoryObject::Monster(CMonster::with_constructor_base_and_type())
        }
        BaseObjectFactoryType::Goods => {
            BaseObjectFactoryObject::Goods(CGoods::with_constructor_base_and_type())
        }
    };
    object.set_object_id(object_id);
    Some(Box::new(object))
}

/// Восстанавливает порядок `CBaseObject::CreateChildObject` без ложной общей
/// модели хранения children.
///
/// `attach` является точной точкой virtual `AddObject`: он вызывается после
/// раннего имени и назначения входного ID, но до восстановления factory-ID и
/// virtual `Load`. Его вызывают только attachment-ветви; player `type=400`
/// с нулевым factory-ID и goods `type=700` с нулевым factory-ID и prefix
/// `NoAdd\0` продолжают инициализацию detached. `load` вызывается во всех
/// успешных ветвях, его результат намеренно отсутствует и потому не может
/// изменить return mapping original-а.
///
/// Как и `CreateObject`, неизвестный type возвращает `Ok(None)` без
/// callback-ов. После `load` входной ID назначается повторно, graphics применяется
/// только при ненулевом значении, а имя копируется вторично, чтобы стереть
/// допустимые мутации `Load`.
pub(crate) fn create_base_object_child<Attach, Load>(
    object_type: i32,
    object_id: i32,
    name: Option<&[u8]>,
    graphics_id: i32,
    mut attach: Attach,
    mut load: Load,
) -> Result<Option<SharedBaseObjectFactoryObject>, BaseObjectChildCreateError>
where
    Attach: FnMut(&SharedBaseObjectFactoryObject),
    Load: FnMut(&mut BaseObjectFactoryObject),
{
    let Some(object) = create_base_object(object_type, object_id) else {
        return Ok(None);
    };
    let child = Rc::new(RefCell::new(BaseObjectTreeNode::from_object(*object)));

    if let Some(name) = name {
        child.borrow_mut().object.set_object_name(name);
    }

    let factory_id = child.borrow().object.object_id();
    child.borrow_mut().object.set_object_id(object_id);

    let attach_to_parent = match object_type {
        400 => factory_id != 0,
        700 if factory_id == 0 => match name {
            Some(name) => !name.starts_with(b"NoAdd\0"),
            None => return Err(BaseObjectChildCreateError::MissingGoodsNameForNoAddProbe),
        },
        _ => true,
    };
    if attach_to_parent {
        attach(&child);
    }

    let mut child_mut = child.borrow_mut();
    child_mut.object.set_object_id(factory_id);
    load(&mut child_mut.object);
    child_mut.object.set_object_id(object_id);
    if graphics_id != 0 {
        child_mut.object.set_object_graphics_id(graphics_id);
    }
    if let Some(name) = name {
        child_mut.object.set_object_name(name);
    }
    drop(child_mut);

    Ok(Some(child))
}

/// Scalar/base-часть исходного `CBaseObject`.
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
    /// Создаёт scalar-начальные состояния точного `CBaseObject` constructor-а.
    ///
    /// Father и child-list живут в `BaseObjectTreeNode`, где их начальные
    /// `None`/empty и destruction-order materialизованы отдельным owner-ом.
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
        let prefix = legacy_c_string_prefix(name);
        self.name.clear();
        self.name.extend_from_slice(prefix);
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

/// Возвращает значимую C-string часть входного безопасного byte-slice.
fn legacy_c_string_prefix(name: &[u8]) -> &[u8] {
    let prefix_len = name
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(name.len());
    &name[..prefix_len]
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
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:280
// RVA: 0x000D5420
// ADDRESS: 004d5420
// PROTOTYPE: void __thiscall DeleteChildObject(CBaseObject * param_1)
//
// Реализовано `BaseObjectTreeNode::delete_child`: сначала удаляются все
// pointer-equal entries parent-list, затем parent отпускает strong ownership.
// Безопасный внешний `Rc` не становится dangling alias старого delete.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::DeleteChildObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:291
// RVA: 0x000D5440
// ADDRESS: 004d5440
// PROTOTYPE: void __thiscall DeleteChildObject(long param_1, long param_2, CGUID * param_3)
//
// Реализовано `BaseObjectTreeNode::delete_child_by_type_and_id`: exact lookup
// читает только type/ID, затем вызывает pointer-вариант для первого list hit.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::CreateObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:157
// RVA: 0x000D5470
// ADDRESS: 004d5470
// PROTOTYPE: CBaseObject * __cdecl CreateObject(long param_1, long param_2)
//
// IMPLEMENTED_OWNER: `create_base_object` выше сохраняет пять literal
// type-ветвей и post-constructor ID assignment. `Box<BaseObjectFactoryObject>`
// заменяет только erased base-pointer/vtable безопасным tagged ownership.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CBaseObject::AddToByteArray` RVA `0x000D55C0` находится выше;
// `Vec` сохраняет append-порядок и wire bytes без STL storage.

// ============================================================================
// FUNCTION: CBaseObject::FindChildObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:84
// RVA: 0x000D5620
// ADDRESS: 004d5620
// PROTOTYPE: bool __thiscall FindChildObject(CBaseObject * param_1)
//
// Реализовано `BaseObjectTreeNode::find_child`: `Rc::ptr_eq` сохраняет pointer
// identity без unsafe raw pointer, а порядок списка для bool не наблюдается.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::FindChildObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:100
// RVA: 0x000D5650
// ADDRESS: 004d5650
// PROTOTYPE: CBaseObject * __thiscall FindChildObject(long param_1, long param_2, CGUID * param_3)
//
// Реализовано `BaseObjectTreeNode::find_child_by_type_and_id`: GUID-аргумент
// в exact теле не читается, а `Vec` сохраняет первый list-order hit.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::RecursiveFindObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:115
// RVA: 0x000D5680
// ADDRESS: 004d5680
// PROTOTYPE: CBaseObject * __thiscall RecursiveFindObject(long param_1, long param_2)
//
// Реализовано `BaseObjectTreeNode::recursive_find_by_type_and_id`: root
// проверяется до children, затем используется exact preorder списка.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::RecursiveFindObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:136
// RVA: 0x000D56D0
// ADDRESS: 004d56d0
// PROTOTYPE: CBaseObject * __thiscall RecursiveFindObject(long param_1, char * param_2)
//
// Реализовано `BaseObjectTreeNode::recursive_find_by_type_and_name`: сравнение
// выполняется по C-string prefix, затем по exact preorder tree.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::BoardCast
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:400
// RVA: 0x000D5750
// ADDRESS: 004d5750
// PROTOTYPE: void __thiscall BoardCast(long param_1, long param_2)
//
// Реализовано `BaseObjectTreeNode::broadcast_children`: callback представляет
// virtual child-slot и получает два signed long без изменения. Original
// обходил list напрямую; snapshot Rust намеренно устраняет только reentrant
// invalidated-iterator/lifetime defect, сохраняя порядок children на входе.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::RemoveObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:321
// RVA: 0x000D5780
// ADDRESS: 004d5780
// PROTOTYPE: void __thiscall RemoveObject(CBaseObject * param_1)
//
// Реализовано `BaseObjectTreeNode::remove_child`: `Vec::retain` удаляет все
// pointer-equal nodes, как `std::list::remove`, и не меняет child father.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::CBaseObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:22
// RVA: 0x000D5790
// ADDRESS: 004d5790
// PROTOTYPE: undefined __thiscall CBaseObject(void)
//
// Реализовано `CBaseObject::with_reached_constructor_defaults` вместе с
// `BaseObjectTreeNode::from_object`: scalar/GUID/name/include-child получают
// доказанные defaults, а empty children и null father — safe `Vec`/`Weak`.
// MSVC list allocation и vtable — только ABI/CRT plumbing и не переносятся.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// VERIFIED_DISASSEMBLY: `CBaseObject::DecordFromByteArray` RVA `0x000D5820`
// находится выше; normal return `true` подтверждён `mov al, 1` по `0x004D589D`.

// VERIFIED_DISASSEMBLY: `CBaseObject::~CBaseObject` RVA `0x000D5D80` ниже не
// вызывает наблюдаемый `AddPlayerList`: call по `0x004D5DE9` приходит в один
// `ret` по `0x00401000`. `BaseObjectTreeNode::Drop` materialизует доказанный
// порядок удаления children до standard Rust cleanup base/factory ownership.

// IMPLEMENTED: `CreateChildObject` RVA `0x000D58B0` передаёт child в
// `AddObject` до virtual `Load`, кроме player ID 0 и точного goods
// `(ID 0, "NoAdd\0")`; результат `Load` игнорируется, а ID/graphics/name затем
// назначаются в доказанном порядке. `BaseObjectTreeNode` теперь задаёт concrete
// storage/father, а explicit callback остаётся virtual attachment-границей.
// BLOCKED_MISSING_FACT: достижим ли `CGoods(type=700, id=0, name=nullptr)` и
// как exact процесс наблюдаемо завершает его безусловное сравнение name?

// VERIFIED_DISASSEMBLY: в exact World EXE нет project-call-site
// `CreateChildObject` RVA `0x000D58B0`; 17 ссылок являются vtable-ячейками, а
// все 27 candidate-инструкций `call [register + 0x28]` принадлежат другим
// классам. Rust поэтому передаёт attachment explicit callback-ом и возвращает
// safe shared handle; concrete tree-storage задаёт `BaseObjectTreeNode`.

// ============================================================================
// FUNCTION: CBaseObject::CreateChildObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:204
// RVA: 0x000D58B0
// ADDRESS: 004d58b0
// PROTOTYPE: CBaseObject * __thiscall CreateChildObject(long param_1, long param_2, char * param_3, long param_4)
//
// Реализовано `create_base_object_child`: его `Rc<RefCell<tagged owner>>`
// заменяет erased returned pointer, `attach` воспроизводит virtual `AddObject`
// до `Load`, а `load` намеренно не возвращает значение, поскольку EXE его
// игнорирует. Для default `AddObject` callback вызывает
// `BaseObjectTreeNode::add_child`, который хранит concrete parent/child и
// фиксирует weak father. `CGoods(id=0, name=nullptr)` вместо
// старого null-dereference даёт typed error без дальнейших side effects;
// короткое имя безопасно не совпадает с `NoAdd\0`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::AddObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:340
// RVA: 0x000D5AD0
// ADDRESS: 004d5ad0
// PROTOTYPE: void __thiscall AddObject(CBaseObject * param_1)
//
// Реализовано `BaseObjectTreeNode::add_child`: assignment weak father идёт до
// append strong child. Duplicate insertion сохранён; cycle безопасно
// отклоняется до мутации как внутренний lifetime defect original-а.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::DeleteAllChildObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:303
// RVA: 0x000D5B90
// ADDRESS: 004d5b90
// PROTOTYPE: void __thiscall DeleteAllChildObject(CBaseObject * param_1)
//
// Реализовано `BaseObjectTreeNode::delete_all_children_except`: snapshot
// children и порядок default delete сохранены; safe aliases не инвалидируются.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::DgFindObjectsByTypes
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:360
// RVA: 0x000D5C50
// ADDRESS: 004d5c50
// PROTOTYPE: void __thiscall DgFindObjectsByTypes(long param_1, _func_long_long_long * param_2, long param_3)
//
// Реализовано `BaseObjectTreeNode::dg_find_objects_by_type`: snapshot и
// list-order сохранены; callback вызывается только для direct child
// совпавшего type, иначе поиск рекурсивно продолжается из child. Father не
// фильтруется: такая дополнительная проверка из C++ reference противоречит raw.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::AI
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:381
// RVA: 0x000D5CF0
// ADDRESS: 004d5cf0
// PROTOTYPE: void __thiscall AI(void)
//
// Реализовано `BaseObjectTreeNode::run_child_ai`: snapshot direct children
// создаётся перед первым callback, пустой list остаётся no-op, а callback
// представляет virtual AI child-slot в сохранённом list-order.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::~CBaseObject
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\baseobject.cpp:33
// RVA: 0x000D5D80
// ADDRESS: 004d5d80
// PROTOTYPE: void __thiscall ~CBaseObject(void)
//
// Реализовано `Drop for BaseObjectTreeNode`: snapshot list сохраняет порядок
// `DeleteAllChildObject(nullptr)`, перед release child из parent-list удаляются
// все pointer-equal entries, а `CBaseObject` после этого освобождается обычным
// Rust ownership. Safe external `Rc` не становится dangling alias; duplicate
// raw pointer больше не вызывает второй delete. Base-vtable, MSVC list/string
// allocator и пустой final call не являются наблюдаемой семантикой.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//












// COMPONENT_VARIANT_END: WorldServer
