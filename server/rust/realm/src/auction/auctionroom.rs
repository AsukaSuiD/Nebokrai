//! Параметры поиска `CAuctionRoom::stPlayerOptNode` из `auctionroom.h`.
//!
//! Default обнуляет player/page/name и low level, включает own-items, задаёт
//! money type `1`, upper level `999` и wildcard weapon type `-1`. Wire хранит
//! 71 DWORD в порядке player, page, low/up/use-self/money/weapon, затем name.
//! `[u8; 256]` сохраняет fixed-buffer и первый NUL без объявления старого ABI.

/// Параметры поиска игрока в точных начальных значениях.
#[derive(Clone, Eq, PartialEq)]
pub struct PlayerOptNode {
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
    pub const fn from_search_request(
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
    pub const fn player_id(&self) -> u32 {
        self.player_id
    }

    /// Возвращает сохранённый zero-based номер страницы.
    pub const fn current_page(&self) -> u32 {
        self.current_page
    }

    /// Перезаписывает только сохранённый номер страницы.
    pub fn set_current_page(&mut self, current_page: u32) {
        self.current_page = current_page;
    }

    /// Возвращает signed нижнюю границу уровня.
    pub const fn low_level(&self) -> i32 {
        self.low_level
    }

    /// Возвращает signed верхнюю границу уровня.
    pub const fn up_level(&self) -> i32 {
        self.up_level
    }

    /// Возвращает требуемый тип валюты.
    pub const fn money_type(&self) -> i32 {
        self.money_type
    }

    /// Возвращает требуемый тип товара либо wildcard `-1`.
    pub const fn weapon_type(&self) -> i32 {
        self.weapon_type
    }

    /// Возвращает фиксированный byte-оригинал поисковый буфер имени.
    pub const fn goods_name(&self) -> &[u8; 256] {
        &self.goods_name
    }
}
