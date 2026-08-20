//! Общий token-scan `ReadTo` из `public/readwrite.cpp`.
//!
//! Статус владельца: `IMPLEMENTED` для четырёх совпадающих вариантов
//! `ReadTo`; World `Read` локально остаётся `BLOCKED_MISSING_FACT` до первого
//! живого владельца его formatted `unsigned long`.
//!
//! Точные пары и RVA:
//! - Auth `AuthServer/authserver.exe + AuthServer/authserver.pdb`,
//!   `0x000158B0`, SHA-256
//!   `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15` /
//!   `26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5`;
//! - Billing `BillingServer/billingserver.exe + BillingServer/billingserver.pdb`,
//!   `0x000132D0`,
//!   `FA32E3C043CB49965686129696A4EB34B733ACA1D60CAF57D369F97D5E68FB19` /
//!   `F900CD0330BEFF32AC071B107AB653FD403CD18746896B3C0187C5751ACA0B21`;
//! - Login `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`,
//!   `0x00020AC0`,
//!   `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876` /
//!   `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`;
//! - World `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//!   `0x0007A330`,
//!   `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1` /
//!   `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//!
//! Исходные пути PDB:
//! `h:\fengyun\fy_russia\src\public\readwrite.cpp`,
//! `d:\complite_version\fengyun_russia\trunk\public\readwrite.cpp` и
//! `e:\svn\fengyun_russia_dev\public\readwrite.cpp`.
//!
//! Все варианты извлекали whitespace-токены до точного совпадения с искомой
//! байтовой строкой и возвращали `false` при EOF, ошибке stream либо при
//! встрече точного `<end>`. `Iterator` и borrowed byte slices заменяют
//! `std::istream/std::string`; кодировка и содержимое токенов не
//! интерпретируются. Старые `std::_Tree`, allocator, SEH/unwind и deleting-
//! destructor блоки удалены как linker/compiler/STL noise; полный экспорт
//! остаётся в истории Git.

/// Продвигает token-stream до точного маркера, но не проходит через `<end>`.
pub(crate) fn read_to<'a>(tokens: &mut impl Iterator<Item = &'a [u8]>, expected: &[u8]) -> bool {
    for token in tokens {
        if token == expected {
            return true;
        }
        if token == b"<end>" {
            return false;
        }
    }
    false
}

// BLOCKED_MISSING_FACT: World `Read` RVA 0x0007A490 сначала вызывал
// `ReadTo(stream, marker)`, затем выполнял `stream >> unsigned long`, но
// возвращал true независимо от успеха второго extraction:
// `if (ReadTo(stream, marker)) { stream >> value; return true; }`.
// До живого call site не выбираются Rust-реакция на malformed число и форма
// результата; отдельный generic parser ради неиспользуемой функции не создаётся.
