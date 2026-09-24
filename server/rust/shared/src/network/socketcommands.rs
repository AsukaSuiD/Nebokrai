//! Потокобезопасная очередь сокетных команд, восстановленная из
//! `nets/socketcommands.cpp`.
//!
//! Статус владельца: `IMPLEMENTED`.
//!
//! Точные варианты и адреса исходных методов:
//! - `AuthServer/authserver.exe + AuthServer/authserver.pdb` (SHA-256 EXE
//!   `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15`, PDB
//!   `26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5`):
//!   `GetSize` RVA `0x00014BC0`, конструктор `0x00014E60`, `Clear`
//!   `0x00014E80`, деструктор `0x00015090`, `Push_Back` `0x00015730`,
//!   `CopyAllCommand` `0x00015830`;
//! - `BillingServer/billingserver.exe + BillingServer/billingserver.pdb`
//!   (SHA-256 EXE
//!   `FA32E3C043CB49965686129696A4EB34B733ACA1D60CAF57D369F97D5E68FB19`, PDB
//!   `F900CD0330BEFF32AC071B107AB653FD403CD18746896B3C0187C5751ACA0B21`):
//!   соответственно `0x0000E4D0`, `0x0000E770`, `0x0000E790`, `0x0000E9A0`,
//!   `0x0000F040`, `0x0000F140`;
//! - `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb` (SHA-256 EXE
//!   `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876`, PDB
//!   `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`):
//!   `GetSize` `0x0006D8C0`, `Pop_Front` `0x0006D920`, конструктор
//!   `0x0006D9A0`, `Clear` `0x0006D9C0`, деструктор `0x0006DBD0`,
//!   `Push_Front` `0x0006E3D0`, `Push_Back` `0x0006E410`,
//!   `AddCommandsQueueToFront` `0x0006E450`, `CopyAllCommand` `0x0006E550`;
//! - `MiscServer/miscserver.exe + MiscServer/miscserver.pdb` (SHA-256 EXE
//!   `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65`, PDB
//!   `ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7`):
//!   соответственно `0x00013B00`, `0x00013B60`, `0x00013BE0`, `0x00013C00`,
//!   `0x00013E10`, `0x00014610`, `0x00014650`, `0x00014690`, `0x00014790`;
//! - `GameServer/gameserver.exe + GameServer/GameServer.pdb` (SHA-256 EXE
//!   `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`, PDB
//!   `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`):
//!   соответственно `0x0001B910`, `0x0001B930`, `0x0001B9B0`, `0x0001B9D0`,
//!   `0x0001BBE0`, `0x0001C400`, `0x0001C440`, `0x0001C480`, `0x0001C580`;
//! - `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb` (SHA-256 EXE
//!   `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//!   `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`):
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
//! текущими; Rust принимает её по значению, исключая двойное владение. Огромный
//! пласт специализаций STL, деревьев, строк, unwind/catch и чужих cleanup-тел,
//! ошибочно приписанный экспортером этому `.cpp`, не относится к владельцу и
//! удалён как compiler/library noise.

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
