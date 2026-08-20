#pragma once

#include "cgoods.h"

#include <cstddef>
#include <cstdint>
#include <map>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <string_view>

/*
 * Исходный владелец: WorldServer/appworld/goods/cgoodsfactory.cpp / .h.
 * PDB подтверждает единый registry по numeric index и индексы трёх валют.
 * Старые глобальные raw pointers заменены owning map. Парсер setup-файла
 * остаётся отдельной границей: фабрика принимает уже проверенные свойства.
 */
class CGoodsFactory
{
public:
    bool Register(std::uint32_t id, CGoodsBaseProperties properties);
    void Clear() noexcept;

    [[nodiscard]] const CGoodsBaseProperties* Query(std::uint32_t id) const noexcept;
    [[nodiscard]] std::optional<std::uint32_t> QueryByOriginalName(std::string_view name) const;
    [[nodiscard]] std::unique_ptr<CGoods> Create(std::uint32_t id, bool rollModifiers = true) const;
    [[nodiscard]] std::unique_ptr<CGoods> Unserialize(std::span<const std::uint8_t> input,
                                                     std::size_t& offset) const;

    void SetCurrencyIndices(std::uint32_t goldCoin,
                            std::uint32_t yuanBao,
                            std::uint32_t jiFen) noexcept;
    [[nodiscard]] std::uint32_t GoldCoinIndex() const noexcept { return m_GoldCoinIndex; }
    [[nodiscard]] std::uint32_t YuanBaoIndex() const noexcept { return m_YuanBaoIndex; }
    [[nodiscard]] std::uint32_t JiFenIndex() const noexcept { return m_JiFenIndex; }

private:
    std::map<std::uint32_t, CGoodsBaseProperties> m_Properties;
    std::map<std::string, std::uint32_t, std::less<>> m_ByOriginalName;
    std::uint32_t m_GoldCoinIndex{};
    std::uint32_t m_YuanBaoIndex{};
    std::uint32_t m_JiFenIndex{};
};
