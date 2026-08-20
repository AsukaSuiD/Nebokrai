#pragma once

#include "../container/ccontainer.h"
#include "../../../public/guid.h"

#include <cstdint>
#include <optional>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/listener/cseekgoodslistener.cpp/.h.
 * Listener собирает GUID товаров одного base-property. Исходное имя сначала
 * разрешает CGoodsFactory; здесь target задаётся уже стабильным ID, поэтому
 * обход не зависит от глобального singleton-а фабрики.
 */
class CSeekGoodsListener final : public CContainerListener {
public:
    void SetTarget(std::uint32_t basePropertyId) noexcept { m_Target = basePropertyId; m_Goods.clear(); }
    void OnTraversingContainer(CContainer& container, CBaseObject& object) override;
    [[nodiscard]] const std::vector<CGUID>& Goods() const noexcept { return m_Goods; }
private:
    std::optional<std::uint32_t> m_Target;
    std::vector<CGUID> m_Goods;
};
