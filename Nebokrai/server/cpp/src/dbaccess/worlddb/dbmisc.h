#pragma once

#include "rssetup.h"
#include "../../public/auctionroom/auctionnode.h"

#include <cstdint>
#include <deque>
#include <mutex>
#include <vector>

/*
 * Исходный владелец: dbaccess/worlddb/dbmisc.cpp/.h.
 * Две очереди DbNote, лимит output-batch 8 и постраничная загрузка auction по
 * owner подтверждены Nworldserver.exe/PDB и Rust. Внешние send/gameplay-эффекты
 * не спрятаны в DB-owner: MainLoop снимает typed note и решает переход.
 */
enum class DbMiscOperation {
    None, InsertItem, InsertItemOk, InsertItemError, ModifyStateSeller,
    ModifyStateSellerOk, ModifyStateBuyer, ModifyStateBuyerOk,
    ModifyStateBuyerError, ReadAuction, ReadAuctionResult,
    DeleteItemSuccess, DeleteItemBack, ModifyMoney,
};

struct DbMiscNote {
    DbMiscOperation operation{DbMiscOperation::None};
    Auction::CGoodsNode goods;
    std::int32_t playerId{-1};
    std::int32_t money{};
};

class CDBMisc {
public:
    static constexpr std::size_t OutputBatchLimit = 8;
    void PushInput(DbMiscNote note, bool front = false);
    void PushOutput(DbMiscNote note);
    [[nodiscard]] std::vector<DbMiscNote> PopInput(std::size_t limit);
    [[nodiscard]] std::vector<DbMiscNote> PopOutput(std::size_t limit = OutputBatchLimit);
    [[nodiscard]] std::size_t InputSize() const;
    [[nodiscard]] std::size_t OutputSize() const;

    [[nodiscard]] static WorldDbResult LoadAuctionOwners(IWorldDbExecutor&);
    [[nodiscard]] static WorldDbResult LoadAuctionByOwner(IWorldDbExecutor&, std::int32_t ownerId,
                                                          std::int32_t state, std::int32_t limit);
    [[nodiscard]] static bool DeleteAuctionGoods(IWorldDbExecutor&, const CGUID&);
    [[nodiscard]] static bool ClearPlayerMoney(IWorldDbExecutor&, std::int32_t playerId);

private:
    static std::vector<DbMiscNote> Pop(std::deque<DbMiscNote>&, std::mutex&, std::size_t);
    mutable std::mutex m_InputMutex;
    mutable std::mutex m_OutputMutex;
    std::deque<DbMiscNote> m_Input;
    std::deque<DbMiscNote> m_Output;
};
