#include "honorranks.h"

#include <stdexcept>

std::vector<HonorRankEntry>& CHonorRanks::Current(const std::size_t type,const std::size_t country)
{if(type>=4||country>=4)throw std::out_of_range("недопустимая таблица почётного ранга");return m_Current[type][country];}
const std::vector<HonorRankEntry>& CHonorRanks::Current(const std::size_t type,const std::size_t country) const
{if(type>=4||country>=4)throw std::out_of_range("недопустимая таблица почётного ранга");return m_Current[type][country];}
