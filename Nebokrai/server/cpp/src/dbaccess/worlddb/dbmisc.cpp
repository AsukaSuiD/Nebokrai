#include "dbmisc.h"

#include <algorithm>
#include <array>
#include <utility>

void CDBMisc::PushInput(DbMiscNote note, const bool front)
{
    std::scoped_lock lock(m_InputMutex);
    if (front) m_Input.push_front(std::move(note));
    else m_Input.push_back(std::move(note));
}

void CDBMisc::PushOutput(DbMiscNote note)
{
    std::scoped_lock lock(m_OutputMutex);
    m_Output.push_back(std::move(note));
}

std::vector<DbMiscNote> CDBMisc::Pop(std::deque<DbMiscNote>& source,
                                     std::mutex& mutex,
                                     const std::size_t limit)
{
    std::scoped_lock lock(mutex);
    std::vector<DbMiscNote> result;
    const std::size_t count = std::min(limit, source.size());
    result.reserve(count);
    for (std::size_t index = 0; index < count; ++index) {
        result.push_back(std::move(source.front()));
        source.pop_front();
    }
    return result;
}

std::vector<DbMiscNote> CDBMisc::PopInput(const std::size_t limit)
{
    return Pop(m_Input, m_InputMutex, limit);
}

std::vector<DbMiscNote> CDBMisc::PopOutput(const std::size_t limit)
{
    return Pop(m_Output, m_OutputMutex, limit);
}

std::size_t CDBMisc::InputSize() const
{
    std::scoped_lock lock(m_InputMutex);
    return m_Input.size();
}

std::size_t CDBMisc::OutputSize() const
{
    std::scoped_lock lock(m_OutputMutex);
    return m_Output.size();
}

WorldDbResult CDBMisc::LoadAuctionOwners(IWorldDbExecutor& database)
{
    return database.Execute({"SELECT DISTINCT dwowerid FROM Auction WITH (NOLOCK)", {}});
}

WorldDbResult CDBMisc::LoadAuctionByOwner(IWorldDbExecutor& database,
                                          const std::int32_t ownerId,
                                          const std::int32_t state,
                                          const std::int32_t limit)
{
    return database.Execute({
        "SELECT TOP (@P3) a.*,b.type,b.modifierValue1,b.modifierValue2 "
        "FROM Auction AS a WITH (NOLOCK) LEFT JOIN AuctionGoods AS b WITH (NOLOCK) ON a.id=b.id "
        "WHERE dwOwerId=@P1 AND GoodsState=@P2 ORDER BY dwOwerId",
        {static_cast<std::int64_t>(ownerId), static_cast<std::int64_t>(state),
         static_cast<std::int64_t>(limit)}});
}

bool CDBMisc::DeleteAuctionGoods(IWorldDbExecutor& database, const CGUID& guid)
{
    std::array<char, 39> text{};
    if (!guid.tostring(text.data(), text.size())) return false;
    return database.Execute({"DELETE FROM Auction WHERE goodsid=@P1", {std::string(text.data())}}).success;
}

bool CDBMisc::ClearPlayerMoney(IWorldDbExecutor& database, const std::int32_t playerId)
{
    return database.Execute({
        "UPDATE AuctionPlayerMoney SET dwmoney=0 WHERE dwplayerid=@P1",
        {static_cast<std::int64_t>(playerId)}}).success;
}
