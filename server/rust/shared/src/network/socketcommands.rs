//! Потокобезопасная очередь сокетных команд, восстановленная из
//! `nets/socketcommands.cpp`.
//!
//! Статус владельца: `IMPLEMENTED`.
//!
//! Точные варианты и адреса исходных методов. Идентификаторы SHA-256 всех
//! перечисленных EXE/PDB зафиксированы в `server/rust/src/manifest/`:
//! - `AuthServer/authserver.exe + AuthServer/authserver.pdb`:
//!   `GetSize` RVA `0x00014BC0`, конструктор `0x00014E60`, `Clear`
//!   `0x00014E80`, деструктор `0x00015090`, `Push_Back` `0x00015730`,
//!   `CopyAllCommand` `0x00015830`;
//! - `BillingServer/billingserver.exe + BillingServer/billingserver.pdb`:
//!   соответственно `0x0000E4D0`, `0x0000E770`, `0x0000E790`, `0x0000E9A0`,
//!   `0x0000F040`, `0x0000F140`;
//! - `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`:
//!   `GetSize` `0x0006D8C0`, `Pop_Front` `0x0006D920`, конструктор
//!   `0x0006D9A0`, `Clear` `0x0006D9C0`, деструктор `0x0006DBD0`,
//!   `Push_Front` `0x0006E3D0`, `Push_Back` `0x0006E410`,
//!   `AddCommandsQueueToFront` `0x0006E450`, `CopyAllCommand` `0x0006E550`;
//! - `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`:
//!   соответственно `0x00013B00`, `0x00013B60`, `0x00013BE0`, `0x00013C00`,
//!   `0x00013E10`, `0x00014610`, `0x00014650`, `0x00014690`, `0x00014790`;
//! - `GameServer/gameserver.exe + GameServer/GameServer.pdb`:
//!   соответственно `0x0001B910`, `0x0001B930`, `0x0001B9B0`, `0x0001B9D0`,
//!   `0x0001BBE0`, `0x0001C400`, `0x0001C440`, `0x0001C480`, `0x0001C580`;
//! - `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`:
//!   соответственно `0x0002AEB0`, `0x0002AF10`, `0x0002AF90`, `0x0002AFB0`,
//!   `0x0002B1C0`, `0x0002B9E0`, `0x0002BA20`, `0x0002BA60`, `0x0002BB60`.
//!
//! Исходные пути PDB:
//! `h:\fengyun\fy_russia\src\nets\socketcommands.cpp`,
//! `d:\complite_version\fengyun_russia\trunk\nets\socketcommands.cpp` и
//! `e:\svn\fengyun_russia_dev\nets\socketcommands.cpp`.
//!
//! Во всех полных вариантах контракт совпадает: push в начало либо конец,
//! извлечение из начала, размер под тем же lock, очистка, атомарная передача всех
//! элементов и вставка перед уже ожидающими командами с сохранением порядка
//! обеих очередей. Auth и Billing не содержат неиспользованных в этих EXE
//! `Pop_Front`, `Push_Front` и `AddCommandsQueueToFront`, но остальные методы
//! совпадают с полными вариантами того же source-файла.
//!
//! Точный PDB дополнительно фиксирует исходный `eSocketOperaType`:
//! `ADD=0`, `CDKEYJOIN=1`, `PLAYERJOIN=2`, `DELBYSOCKETID=3`,
//! `QUITBYSOCKETID=4`, `QUITBYMAPID=5`, `QUITBYMAPSTR=6`, `QUITALL=7`,
//! `RECIEVE=8`, `SENDTOSOCKET=9`, `SENDTOMAPID=10`, `SENDTOMAPSTR=11`,
//! `SENDALL=12`, `ONRECEIVE=13`, `ONSEND=14`, `ONCLOSE=15`,
//! `ONCONNECT=16`, `SENDEND=17`. `tagSocketOper` занимает 24 байта в старом
//! 32-битном ABI: `OperaType` `+0`, `lSocketID` `+4`, `pStrID` `+8`, `pBuf`
//! `+12`, `lNum1` `+16`, `lNum2` `+20`.
//!
//! Конкретная Rust-форма `tagSocketOper` намеренно не выбирается этим owner:
//! смысл полей и допустимые их сочетания доказывают производители и потребители
//! в `clients.rs` и `servers.rs`. Generic `T` не даёт очереди превратить этот
//! layout в преждевременный общий framework. Будущий конкретный тип обязан
//! владеть выделенными ему строкой и buffer; тогда обычный `Drop` заменяет
//! ручные ветви `Clear`, `operator_delete` и удаление самого указателя.
//!
//! `parking_lot::Mutex<VecDeque<T>>` заменяет `CRITICAL_SECTION`, старый deque и
//! его allocator. Mutex не вводит отсутствующее poisoning, `VecDeque` сохраняет
//! порядок, а передача `T` — владение. Старый nullable `Push_* -> false` не
//! переносится: отсутствие команды разбирает вызывающий владелец, сохранённый
//! `T` всегда существует. Возвращаемый `bool` ни один проверенный call site не
//! использует.
//!
//! `CopyAllCommand` фактически передавал все указатели наружу и обнулял исходный
//! deque; Rust возвращает владеющий `VecDeque<T>`. `AddCommandsQueueToFront`
//! принимал локальную очередь повторной отправки и копировал её указатели перед
//! текущими; Rust принимает её по значению, исключая двойное владение.

//! Экземпляр очереди хранит владелец процесса/направления;
//! Shared несёт только типы записей и атомарность операций.

use std::collections::VecDeque;
use std::mem;

use parking_lot::Mutex;

/// Владеющая двусторонняя очередь команд с атомарностью исходного
/// `CSocketCommands`.
///
/// `Drop` элемента может выполняться под mutex при [`Self::clear`]. Конкретный
/// тип команды не должен повторно входить в тот же экземпляр очереди из своего
/// деструктора; исходный `tagSocketOper` такого поведения не имел.
pub struct CSocketCommands<T> {
    commands: Mutex<VecDeque<T>>,
}

impl<T> CSocketCommands<T> {
    /// Создаёт пустую очередь с готовым примитивом синхронизации.
    pub fn new() -> Self {
        Self {
            commands: Mutex::new(VecDeque::new()),
        }
    }

    /// Возвращает размер как 32-битный Windows `long`, сохраняя его младшие
    /// биты при теоретическом переполнении.
    pub fn get_size(&self) -> i32 {
        self.commands.lock().len() as u32 as i32
    }

    /// Передаёт владение командой и ставит её перед всеми ожидающими.
    pub fn push_front(&self, command: T) {
        self.commands.lock().push_front(command);
    }

    /// Передаёт владение командой и ставит её после всех ожидающих.
    pub fn push_back(&self, command: T) {
        self.commands.lock().push_back(command);
    }

    /// Извлекает первую команду либо возвращает `None` для пустой очереди.
    pub fn pop_front(&self) -> Option<T> {
        self.commands.lock().pop_front()
    }

    /// Уничтожает все ожидающие команды под тем же mutex.
    pub fn clear(&self) {
        self.commands.lock().clear();
    }

    /// Атомарно опустошает очередь и передаёт все команды в исходном порядке.
    pub fn take_all(&self) -> VecDeque<T> {
        mem::take(&mut *self.commands.lock())
    }

    /// Ставит всю переданную очередь перед уже ожидающими командами.
    ///
    /// Порядок внутри `front` и внутри текущей очереди сохраняется.
    pub fn prepend(&self, mut front: VecDeque<T>) {
        let mut commands = self.commands.lock();
        front.append(&mut commands);
        *commands = front;
    }
}

impl<T> Default for CSocketCommands<T> {
    fn default() -> Self {
        Self::new()
    }
}
