#pragma once

#include <cstddef>
#include <cstdint>
#include <deque>
#include <mutex>
#include <vector>

/*
 * Owner: loginserver/servlogqueue.cpp / servlogqueue.h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Исходный путь PDB:
 * d:\complite_version\fengyun_russia\trunk\server\loginserver\loginserver\servlogqueue.cpp
 *
 * Подтверждённые RVA: size 0x0001F6F0, pop 0x0001F780,
 * clear 0x0001F810, push 0x0001FB50. Исходный owner хранил указатели на
 * отдельные ServLog в std::deque под одной CRITICAL_SECTION: push глубоко
 * копировал четыре поля и добавлял запись в хвост, pop снимал голову, clear
 * удалял всё под тем же lock.
 *
 * Здесь Windows CRITICAL_SECTION, ручные new/delete и deque указателей не
 * воспроизводятся: стандартные std::mutex + std::deque<ServLog> дают ту же
 * существенную FIFO/lifetime-семантику. Description остаётся byte-exact и не
 * перекодируется. Ограничений ёмкости или длины строки исходный достигнутый
 * контракт не содержал; защитные лимиты старого Linux-донора сюда не перенесены.
 */
namespace Login
{
struct ServLog
{
    std::uint32_t sourceIp{};
    std::int32_t serverType{};
    std::int32_t serverNumber{};
    std::vector<std::uint8_t> description;
};

class ServLogQueue
{
public:
    ServLogQueue() = default;

    ServLogQueue(const ServLogQueue&) = delete;
    ServLogQueue& operator=(const ServLogQueue&) = delete;

    void Push(ServLog record);
    [[nodiscard]] bool Pop(ServLog& record);
    [[nodiscard]] std::int32_t Size() const;
    void Clear();

private:
    mutable std::mutex m_Mutex;
    std::deque<ServLog> m_Records;
};
}
