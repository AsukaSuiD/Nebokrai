//! Observable traversal-контракты запросов `CServerRegion` исторического
//! GameServer (порция 1). Исходный владелец — `appserver/serverregion.h/.cpp`;
//! точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`.

/// Воспроизводит только observable traversal `m_mNpcs` из decoder RVA
/// `0x000858F0`. MSVC `_Hash::insert` RVA `0x00081A60` начинает с mask/bucket
/// `1/1`, растит один bucket на каждые четыре элемента, группирует linked
/// list по bucket и держит signed `long` keys по возрастанию внутри группы.
pub fn legacy_msvc_npc_hash_traversal(ids: impl IntoIterator<Item = i32>) -> Vec<i32> {
    let mut ids: Vec<_> = ids.into_iter().collect();
    let mut mask = 1_u32;
    let mut bucket_count = 1_u32;
    let mut bucket_vector_len = 9_u32;

    for inserted in 0..ids.len() as u32 {
        if bucket_count <= inserted >> 2 {
            if bucket_count < bucket_vector_len - 1 {
                if mask < bucket_count {
                    mask = mask.wrapping_mul(2).wrapping_add(1);
                }
            } else {
                mask = bucket_vector_len.wrapping_mul(2).wrapping_sub(3);
                bucket_vector_len = bucket_vector_len.wrapping_mul(2).wrapping_sub(1);
            }
            bucket_count = bucket_count.wrapping_add(1);
        }
    }

    ids.sort_by_key(|id| {
        let mut bucket = (*id as u32 ^ 0xdead_beef) & mask;
        if bucket_count <= bucket {
            bucket = bucket.wrapping_sub(1 + (mask >> 1));
        }
        (bucket, *id)
    });
    ids
}
