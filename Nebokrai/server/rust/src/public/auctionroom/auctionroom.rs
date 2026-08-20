//! Параметры поиска `CAuctionRoom::stPlayerOptNode` из `auctionroom.h`.
//!
//! Статус владельца: `IMPLEMENTED`. Исходный владелец PDB:
//! `h:\fengyun\fy_russia\src\public\auctionroom\auctionroom.h:56`; точная пара
//! `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`, конструктор RVA
//! `0x00002F80`. Других компонентных вариантов в сыром корпусе нет.
//!
//! Конструктор обнуляет player/page и 256 байт имени, оставляет нижний уровень
//! нулевым, включает поиск собственных предметов, выбирает money type `1`,
//! задаёт верхний уровень `999` и wildcard weapon type `-1`. Case `0x14ED09`
//! и точный PDB подтверждают физический порядок 71 DWORD: player, page,
//! low/up/use-self/money/weapon и только затем имя. Фиксированный буфер
//! сохраняется как `[u8; 256]`; byte-substring и первый NUL используются
//! достигнутым `IsMatchCondition`, а case `0x14ED09` создаёт полное значение
//! из wire-полей перед передачей `ModifyPlayerSeachCondition`.
//! CRT/compiler noise отсутствует.

/// Параметры поиска игрока в точных начальных значениях.
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct PlayerOptNode {
    player_id: u32,
    current_page: u32,
    low_level: i32,
    up_level: i32,
    use_self: i32,
    money_type: i32,
    weapon_type: i32,
    goods_name: [u8; 256],
}

impl Default for PlayerOptNode {
    fn default() -> Self {
        Self {
            player_id: 0,
            current_page: 0,
            low_level: 0,
            up_level: 999,
            use_self: 1,
            money_type: 1,
            weapon_type: -1,
            goods_name: [0; 256],
        }
    }
}

impl PlayerOptNode {
    /// Создаёт условие из полного набора полей сообщения и сбрасывает page.
    pub(crate) const fn from_search_request(
        player_id: u32,
        low_level: i32,
        up_level: i32,
        use_self: i32,
        money_type: i32,
        weapon_type: i32,
        goods_name: [u8; 256],
    ) -> Self {
        Self {
            player_id,
            current_page: 0,
            low_level,
            up_level,
            use_self,
            money_type,
            weapon_type,
            goods_name,
        }
    }

    /// Возвращает unsigned ключ игрока для search-map.
    pub(super) const fn player_id(&self) -> u32 {
        self.player_id
    }

    /// Возвращает сохранённый zero-based номер страницы.
    pub(super) const fn current_page(&self) -> u32 {
        self.current_page
    }

    /// Перезаписывает только сохранённый номер страницы.
    pub(super) fn set_current_page(&mut self, current_page: u32) {
        self.current_page = current_page;
    }

    /// Возвращает signed нижнюю границу уровня.
    pub(super) const fn low_level(&self) -> i32 {
        self.low_level
    }

    /// Возвращает signed верхнюю границу уровня.
    pub(super) const fn up_level(&self) -> i32 {
        self.up_level
    }

    /// Возвращает требуемый тип валюты.
    pub(super) const fn money_type(&self) -> i32 {
        self.money_type
    }

    /// Возвращает требуемый тип товара либо wildcard `-1`.
    pub(super) const fn weapon_type(&self) -> i32 {
        self.weapon_type
    }

    /// Возвращает фиксированный byte-exact поисковый буфер имени.
    pub(super) const fn goods_name(&self) -> &[u8; 256] {
        &self.goods_name
    }
}
