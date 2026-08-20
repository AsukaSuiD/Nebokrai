//! Пустой DB-адаптер `CRsVillageWar` исторического WorldServer.
//!
//! Статус constructor RVA `0x000F7760` и destructor RVA `0x000F7780` —
//! `IMPLEMENTED` классификацией: отдельная Rust-семантика не требуется. Точная
//! пара `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256
//! EXE `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsvillagewar.cpp`.
//!
//! Exact PDB публикует у concrete-класса только constructor, virtual
//! destructor, deleting-destructor и RTTI/vtable. Constructor вызывает пустой
//! `CMyAdoBase` constructor и меняет vtable; destructor восстанавливает vtable
//! и вызывает base destructor. Ни одного DB-метода класса в linked PDB нет.
//!
//! Точечный disassembly exact `DoSaveData` RVA `0x0001C610` до следующего
//! владельца по `0x0001E330` не содержит вызовов constructor/destructor этого
//! класса, war-строк либо отдельной Village War transaction phase. Найденное в
//! PDB имя `SaveAllVillageWarParam` не имеет выполняемого символа в этой
//! функции и не является основанием придумывать save API.
//!
//! Поэтому ADO/vtable/RTTI, MSVC `deque` helper-ы, unwind и deleting-destructor
//! являются только заменённым library/compiler noise. Rust DB-owner-ы создаются
//! непосредственно для доказанных операций, а обычное владение и `Drop`
//! заменяют пустой lifetime этого адаптера. Сырой псевдокод после классификации
//! удалён; локальных неизвестностей наблюдаемого контракта здесь не осталось.
