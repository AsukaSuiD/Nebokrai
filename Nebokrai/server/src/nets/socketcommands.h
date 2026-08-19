#pragma once

#include <cstdint>
#include <deque>
#include <iterator>
#include <mutex>
#include <optional>
#include <utility>

/*
 * Owner: nets/socketcommands.cpp
 *
 * Источник: точные EXE/PDB AuthServer, LoginServer, BillingServer, MiscServer,
 * GameServer и Nworldserver. Полные варианты подтверждают один контракт:
 * двусторонняя потокобезопасная очередь socket-команд, push в начало/конец,
 * pop из начала, размер под тем же lock, очистка, атомарная передача всего
 * snapshot и постановка локальной очереди перед уже ожидающими командами с
 * сохранением порядка обеих частей.
 *
 * Auth и Billing не содержат неиспользованные Pop_Front, Push_Front и
 * AddCommandsQueueToFront, но присутствующие методы совпадают с полными
 * вариантами Login/Misc/Game/World.
 *
 * Точный PDB фиксирует исходный eSocketOperaType от ADD=0 до SENDEND=17 и
 * старый 32-битный tagSocketOper размером 24 байта. Конкретный современный
 * payload здесь намеренно не материализуется: pBuf/pStrID имели разный смысл у
 * producers/consumers. clients и servers получают собственные владеющие типы
 * команд, а этот owner хранит только доказанную семантику очереди.
 *
 * Старые CRITICAL_SECTION, std::deque<tagSocketOper*>, allocator и ручные
 * ветви delete заменены std::mutex, std::deque<T> и обычным владением C++.
 * eventfd, wake callbacks, лимиты очереди, byte accounting, control reserve и
 * дополнительные World operation 18/19 из старого Linux-донора оригинальными
 * EXE/PDB не подтверждены и сюда не переносятся.
 */
template <typename T>
class CSocketCommands
{
public:
    using Queue = std::deque<T>;

    CSocketCommands() = default;
    ~CSocketCommands()
    {
        Clear();
    }

    CSocketCommands(const CSocketCommands&) = delete;
    CSocketCommands& operator=(const CSocketCommands&) = delete;
    CSocketCommands(CSocketCommands&&) = delete;
    CSocketCommands& operator=(CSocketCommands&&) = delete;

    [[nodiscard]] std::int32_t GetSize() const
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionCommands);

        // Исходный Windows long был 32-битным; сохраняются младшие 32 бита.
        return static_cast<std::int32_t>(static_cast<std::uint32_t>(m_Commands.size()));
    }

    [[nodiscard]] std::optional<T> Pop_Front()
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionCommands);
        if (m_Commands.empty()) {
            return std::nullopt;
        }

        T command = std::move(m_Commands.front());
        m_Commands.pop_front();
        return command;
    }

    void Push_Front(T command)
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionCommands);
        m_Commands.push_front(std::move(command));
    }

    void Push_Back(T command)
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionCommands);
        m_Commands.push_back(std::move(command));
    }

    void AddCommandsQueueToFront(Queue commands)
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionCommands);

        // Переданная очередь остаётся впереди уже накопленной, внутренний
        // порядок обеих частей не меняется.
        commands.insert(commands.end(),
                        std::make_move_iterator(m_Commands.begin()),
                        std::make_move_iterator(m_Commands.end()));
        m_Commands = std::move(commands);
    }

    [[nodiscard]] Queue CopyAllCommand()
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionCommands);

        // Исходный CopyAllCommand передавал все указатели вызывающему и очищал
        // внутренний deque в той же critical section.
        Queue commands = std::move(m_Commands);
        m_Commands.clear();
        return commands;
    }

    void Clear()
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionCommands);
        m_Commands.clear();
    }

private:
    mutable std::mutex m_CriticalSectionCommands;
    Queue m_Commands;
};
