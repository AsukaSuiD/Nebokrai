//! Пустой DB-адаптер `CRsCityWar` исторического WorldServer.
//!
//! Статус constructor RVA `0x000F7710` и destructor RVA `0x000F7730` —
//! `IMPLEMENTED` классификацией: отдельная Rust-семантика не требуется. Точная
//! пара `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256
//! EXE `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\rscitywar.cpp`.
//!
//! Exact PDB публикует у concrete-класса только constructor, virtual
//! destructor, deleting-destructor и RTTI/vtable. Оба тела меняют лишь
//! `CMyAdoBase`/concrete vtable и lifetime пустого base-объекта; DB-методов у
//! linked класса нет.
//!
//! Точечный disassembly exact `DoSaveData` RVA `0x0001C610` до следующего
//! владельца по `0x0001E330` не содержит вызовов constructor/destructor этого
//! класса, war-строк либо отдельной City War transaction phase. PDB-имя
//! `SaveAllCityWarParam` не связано с выполняемым символом этой функции и не
//! доказывает отсутствующий owner-вызов.
//!
//! Поэтому ADO/vtable/RTTI, COM cleanup, unwind и deleting-destructor являются
//! только заменённым library/compiler noise. Rust не получает фиктивный City
//! War DB API: доказанные DB-owner-ы владеют своими операциями напрямую, а
//! обычное владение и `Drop` заменяют пустой lifetime адаптера. Сырой псевдокод
//! после классификации удалён; локальных неизвестностей наблюдаемого контракта
//! здесь не осталось.
