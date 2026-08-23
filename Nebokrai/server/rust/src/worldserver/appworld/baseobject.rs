//! Базовый объект `CBaseObject` из `baseobject.cpp/.h`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Type, ID, graphics ID, GUID и byte-exact имя сериализуются в исходном
//! little-endian порядке; имя завершается единственным NUL. Decoder сохраняет
//! уже присвоенные scalars при поздней ошибке и сдвигает внешний cursor только
//! на фактически прочитанные данные.
//!
//! Child-tree сохраняет порядок списка и необычное владение оригинала:
//! `AddObject` допускает повторы и сначала назначает parent, `RemoveObject`
//! удаляет все совпадения без очистки parent, а recursive delete обходит snapshot
//! списка. `CreateChildObject` сохраняет исключения для player и товара `NoAdd`,
//! порядок `Load`, повторное назначение ID и позднюю запись graphics ID.
//!
//! Tagged factory различает Region, Player, NPC, Monster и Goods без erased
//! pointer. `Rc`/`Weak`, `Vec<u8>` и стандартное владение заменяют raw pointers
//! и STL; циклы, null-name у особого Goods и короткие wire-данные останавливаются
//! до прежнего UB, не меняя штатный путь.

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
/// могло быть присвоено, однако немедленно аварийно завершался до
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
/// существовать в сохранённых объектах, но factory возвращает для них
/// null. Значения подтверждены ветвями.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseObjectFactoryType {
    Region,
    Player,
    Npc,
    Monster,
    Goods,
}

impl BaseObjectFactoryType {
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
/// deleting destructor parent-а. `father` намеренно weak: оригинал pointer не
/// владел parent-ом. Duplicate children допустимы как в original-е, а cycle
/// отвергается до изменения state, поскольку его рекурсивные owner-ы не имели
/// исходного внешнего эффекта, но приводили к internal lifetime defect.
pub(crate) struct BaseObjectTreeNode {
    object: BaseObjectFactoryObject,
    father: Option<std::rc::Weak<RefCell<BaseObjectTreeNode>>>,
    children: Vec<SharedBaseObjectFactoryObject>,
}

pub(crate) type SharedBaseObjectFactoryObject = Rc<RefCell<BaseObjectTreeNode>>;

impl BaseObjectTreeNode {
    fn from_object(object: BaseObjectFactoryObject) -> Self {
        Self {
            object,
            father: None,
            children: Vec::new(),
        }
    }

    pub(crate) fn create(
        object_type: i32,
        object_id: i32,
    ) -> Option<SharedBaseObjectFactoryObject> {
        create_base_object(object_type, object_id)
            .map(|object| Rc::new(RefCell::new(Self::from_object(*object))))
    }

    pub(crate) fn object(&self) -> &BaseObjectFactoryObject {
        &self.object
    }

    pub(crate) fn object_mut(&mut self) -> &mut BaseObjectFactoryObject {
        &mut self.object
    }

    pub(crate) fn father(&self) -> Option<SharedBaseObjectFactoryObject> {
        self.father.as_ref().and_then(std::rc::Weak::upgrade)
    }

    pub(crate) fn children(&self) -> Vec<SharedBaseObjectFactoryObject> {
        self.children.clone()
    }

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

    pub(crate) fn recursive_find_by_type_and_id(
        root: &SharedBaseObjectFactoryObject,
        object_type: i32,
        object_id: i32,
    ) -> Option<SharedBaseObjectFactoryObject> {
        Self::recursive_find(root, &mut |object| {
            object.object_type() == object_type && object.object_id() == object_id
        })
    }

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
 /// оригинал проходит сам `std::list`, не создавая snapshot. Вызванный virtual
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
 /// `parameter`; несовпавший рекурсивно ищется в своём поддереве. Как оригинал,
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
 /// snapshot сохраняет оригинал порядок и разрешает callback-у безопасно менять
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
 // оригинал pointer в original-е приводил бы к dangling delete; `Rc` оставляет
 // это внутреннее повреждение безопасным и освобождает alias один раз.
        let children = self.children.clone();
        for child in children {
            self.children
                .retain(|candidate| !Rc::ptr_eq(candidate, &child));
        }
    }
}

impl BaseObjectFactoryObject {
    pub(crate) const fn object_type(&self) -> i32 {
        match self {
            Self::Region(object) => object.get_type(),
            Self::Player(object) => object.get_type(),
            Self::Npc(object) => object.get_type(),
            Self::Monster(object) => object.get_type(),
            Self::Goods(object) => object.get_type(),
        }
    }

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

/// Создаёт точные пять ветвей `CBaseObject::CreateObject`.
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
/// раннего имени и назначения входного ID, но до присоединения factory-ID и
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

pub(crate) struct CBaseObject {
    object_type: i32,
    id: i32,
    ex_id: CGuid,
    graphics_id: i32,
    name: Vec<u8>,
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

    pub(crate) const fn get_type(&self) -> i32 {
        self.object_type
    }

    pub(crate) const fn factory_type(&self) -> Option<BaseObjectFactoryType> {
        BaseObjectFactoryType::from_wire_value(self.object_type)
    }

    pub(crate) const fn set_type(&mut self, object_type: i32) {
        self.object_type = object_type;
    }

    pub(crate) const fn get_id(&self) -> i32 {
        self.id
    }

    pub(crate) const fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    pub(crate) const fn get_ex_id(&self) -> &CGuid {
        &self.ex_id
    }

    pub(crate) const fn set_ex_id(&mut self, ex_id: &CGuid) {
        self.ex_id = *ex_id;
    }

    pub(crate) const fn get_graphics_id(&self) -> i32 {
        self.graphics_id
    }

    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.graphics_id = graphics_id;
    }

    pub(crate) fn get_name(&self) -> &[u8] {
        &self.name
    }

    pub(crate) fn set_name(&mut self, name: &[u8]) {
        let prefix = legacy_c_string_prefix(name);
        self.name.clear();
        self.name.extend_from_slice(prefix);
    }

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
 // Оригинал сначала сдвигал `long&`, затем читал
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
 // Вспомогательная функция World не знала длину
 // источника и продолжала чтение до NUL. Реакция при
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
 // Вспомогательная функция уже потребила этот байт перед записью за
 // границей локального `char[256]`; достижимость и результат такого
 // повреждения stack не определены.
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

fn legacy_c_string_prefix(name: &[u8]) -> &[u8] {
    let prefix_len = name
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(name.len());
    &name[..prefix_len]
}
