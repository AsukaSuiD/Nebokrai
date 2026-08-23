//! Межрегиональная таблица маршрутизации исторического Miracle.
//!
//! Контракт World `CRegionRouter::AddToByteArray` и
//! `CRegionRouter::LoadRouterSetup` и
//! `CRegionRouter::ChageRegionRouter`:;
//! singleton и Game decoder не входят в этот owner и остаются. Точная пара:
//! Исходный владелец PDB:
//!
//! Wire: signed region count, затем для каждого ordered map node шесть `i32`
//! (`region_id`, вход X/Y, выход X/Y, out-range), signed next count и ordered
//! 12-байтные `next_region_id + X + Y`. Map key отдельно не передаётся: EXE
//! пишет ID из value; это различие сохранено. Параметр `sendSelf` точным
//! serializer-ом не читался. `BTreeMap` и owned values заменяют MSVC tree и
//! raw struct-copy, не меняя signed ordering или compact layout.
//!
//! Route search проверяет наличие обоих map keys, а при совпадении регионов
//! возвращает единственную конечную точку. Для разных регионов EXE выбирает
//! среди ещё не закрытых узлов минимальную hop-distance, при равенстве —
//! меньший signed region key, и обновляет соседей только при строго меньшей
//! дистанции. `BinaryHeap<Reverse<(distance, region)>>` и `BTreeMap` являются
//! safe заменой копии MSVC tree и полного линейного поиска минимума; порядок и
//! tie-break при этом совпадают. Маршрут использует map keys (не дублирующие
//! ID из values), координаты перехода — из `next[next_region_key]`; для каждого
//! промежуточного региона выдаются entry и exit-to-next, для конечного — entry
//! и запрошенные X/Y. Отсутствующий переход даёт `(0, 0)`, как value-initialized
//! временный `tagPOINT` в EXE. `pOut` и out-range эта функция не читает.
//!
//! Loader сначала очищает owner даже при последующей ошибке открытия. Формат
//! whitespace-token based: ignored label + unsigned 32-bit region count; для
//! каждого региона — `label + region ID`, `label + entry X/Y`, `label + exit
//! X/Y`, `label + out-range`, `label + unsigned next count`, затем next count
//! троек `(next region ID, X, Y)` без labels. Хвост игнорируется. Оба MSVC map
//! используют insert-only: первый duplicate key остаётся. `std::fs::read` и
//! byte-token parser заменяют `CRFile + stringstream`. Malformed stream в C++
//! продолжал читать неинициализированные locals; Rust прекращает загрузку с
//! typed error, сохраняя уже подтверждённый prefix и исключая внутренний UB.

use std::cmp::Reverse;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRoutePoint {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionNextNode {
    pub(crate) next_region_id: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRouterNode {
    pub(crate) region_id: i32,
    pub(crate) entry: RegionRoutePoint,
    pub(crate) exit: RegionRoutePoint,
    pub(crate) exit_range: i32,
    pub(crate) next: BTreeMap<i32, RegionNextNode>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRouter {
    nodes: BTreeMap<i32, RegionRouterNode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionRouteStep {
    pub(crate) region_id: i32,
    pub(crate) point: RegionRoutePoint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionRouterChangeOutcome {
    RegionNotFound {
        from_region_found: bool,
        to_region_found: bool,
    },
    Complete(Vec<RegionRouteStep>),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRouterLoadReport {
    pub(crate) declared_regions: u32,
    pub(crate) inserted_regions: u32,
    pub(crate) duplicate_regions: u32,
    pub(crate) duplicate_transitions: u32,
    pub(crate) trailing_tokens: usize,
}

#[derive(Debug)]
pub(crate) enum RegionRouterLoadError {
    Io(io::Error),
    MissingToken { field: &'static str },
    InvalidInteger { field: &'static str, token: Vec<u8> },
}

impl PartialEq for RegionRouterLoadError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(left), Self::Io(right)) => {
                left.kind() == right.kind() && left.to_string() == right.to_string()
            }
            (
                Self::MissingToken { field: left },
                Self::MissingToken { field: right },
            ) => left == right,
            (
                Self::InvalidInteger {
                    field: left_field,
                    token: left_token,
                },
                Self::InvalidInteger {
                    field: right_field,
                    token: right_token,
                },
            ) => left_field == right_field && left_token == right_token,
            _ => false,
        }
    }
}

impl Eq for RegionRouterLoadError {}

impl RegionRouter {
    pub(crate) fn insert_node(
        &mut self,
        map_region_id: i32,
        node: RegionRouterNode,
    ) -> Option<RegionRouterNode> {
        self.nodes.insert(map_region_id, node)
    }

    pub(crate) fn clear(&mut self) {
        self.nodes.clear();
    }

 /// Загружает оригинал World whitespace grammar `RegionRouter.ini`.
    pub(crate) fn load_router_setup(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<RegionRouterLoadReport, RegionRouterLoadError> {
        self.clear();
        let bytes = fs::read(path).map_err(RegionRouterLoadError::Io)?;
        self.load_router_setup_bytes(&bytes)
    }

 /// Та же grammar для уже открытого package-resource WorldServer.
    pub(crate) fn load_router_setup_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<RegionRouterLoadReport, RegionRouterLoadError> {
        self.clear();
        let mut tokens = RegionRouterTokens::new(&bytes);
        tokens.skip("region count label")?;
        let declared_regions = tokens.read_u32("region count")?;
        let mut report = RegionRouterLoadReport {
            declared_regions,
            ..RegionRouterLoadReport::default()
        };

        for _ in 0..declared_regions {
            tokens.skip("region ID label")?;
            let region_id = tokens.read_i32("region ID")?;
            tokens.skip("entry point label")?;
            let entry = RegionRoutePoint {
                x: tokens.read_i32("entry X")?,
                y: tokens.read_i32("entry Y")?,
            };
            tokens.skip("exit point label")?;
            let exit = RegionRoutePoint {
                x: tokens.read_i32("exit X")?,
                y: tokens.read_i32("exit Y")?,
            };
            tokens.skip("exit range label")?;
            let exit_range = tokens.read_i32("exit range")?;
            tokens.skip("transition count label")?;
            let transition_count = tokens.read_u32("transition count")?;
            let mut next = BTreeMap::new();
            for _ in 0..transition_count {
                let next_region_id = tokens.read_i32("next region ID")?;
                let transition = RegionNextNode {
                    next_region_id,
                    x: tokens.read_i32("transition X")?,
                    y: tokens.read_i32("transition Y")?,
                };
                match next.entry(next_region_id) {
                    Entry::Vacant(entry) => {
                        entry.insert(transition);
                    }
                    Entry::Occupied(_) => {
                        report.duplicate_transitions += 1;
                    }
                }
            }

            let node = RegionRouterNode {
                region_id,
                entry,
                exit,
                exit_range,
                next,
            };
            match self.nodes.entry(region_id) {
                Entry::Vacant(entry) => {
                    entry.insert(node);
                    report.inserted_regions += 1;
                }
                Entry::Occupied(_) => {
                    report.duplicate_regions += 1;
                }
            }
        }
        report.trailing_tokens = tokens.remaining();
        Ok(report)
    }

 /// Восстанавливает оригинал World `ChageRegionRouter` с детерминированным
 /// signed-key tie-break исходного `std::map`.
    pub(crate) fn change_region_router(
        &self,
        from_region: i32,
        to_region: i32,
        target: RegionRoutePoint,
    ) -> RegionRouterChangeOutcome {
        let from_region_found = self.nodes.contains_key(&from_region);
        let to_region_found = self.nodes.contains_key(&to_region);
        if !from_region_found || !to_region_found {
            return RegionRouterChangeOutcome::RegionNotFound {
                from_region_found,
                to_region_found,
            };
        }
        if from_region == to_region {
            return RegionRouterChangeOutcome::Complete(vec![RegionRouteStep {
                region_id: to_region,
                point: target,
            }]);
        }

        const INITIAL_ROUTE_VALUE: i32 = 20_000_000;

        let mut distance = self
            .nodes
            .keys()
            .copied()
            .map(|region_id| (region_id, INITIAL_ROUTE_VALUE))
            .collect::<BTreeMap<_, _>>();
        let mut predecessor = BTreeMap::<i32, i32>::new();
        let mut settled = BTreeSet::<i32>::new();
        let mut pending = BinaryHeap::<Reverse<(i32, i32)>>::new();
        distance.insert(from_region, 0);
        pending.push(Reverse((0, from_region)));

        while let Some(Reverse((current_distance, current_region))) = pending.pop() {
            if settled.contains(&current_region)
                || distance.get(&current_region).copied() != Some(current_distance)
            {
                continue;
            }
            settled.insert(current_region);

            let current_node = self
                .nodes
                .get(&current_region)
                .expect("pending содержит только ключи RegionRouter");
 // Машина сравнивает destination с `stRouterNode::lRegionId`, хотя
 // путь и lookup-и строит по map key. Несовпадение полей сохраняем.
            if current_node.region_id == to_region {
                break;
            }

            let next_distance = current_distance + 1;
            for next_region in current_node.next.keys().copied() {
                let Some(known_distance) = distance.get_mut(&next_region) else {
                    continue;
                };
                if next_distance < *known_distance {
                    *known_distance = next_distance;
                    predecessor.insert(next_region, current_region);
                    pending.push(Reverse((next_distance, next_region)));
                }
            }
        }

        if !settled.contains(&to_region) {
            return RegionRouterChangeOutcome::Complete(Vec::new());
        }

        let mut reverse_path = vec![to_region];
        let mut current_region = to_region;
        while current_region != from_region {
            let Some(previous_region) = predecessor.get(&current_region).copied() else {
                return RegionRouterChangeOutcome::Complete(Vec::new());
            };
            reverse_path.push(previous_region);
            current_region = previous_region;
        }
        reverse_path.reverse();

        let mut route = Vec::new();
        for (index, region_id) in reverse_path.iter().copied().enumerate() {
            let node = self
                .nodes
                .get(&region_id)
                .expect("reconstructed path содержит только ключи RegionRouter");
            if index == 0 {
                let next_region = reverse_path[index + 1];
                route.push(RegionRouteStep {
                    region_id,
                    point: node.next_point(next_region),
                });
                continue;
            }

            route.push(RegionRouteStep {
                region_id,
                point: node.entry,
            });
            let point = reverse_path
                .get(index + 1)
                .copied()
                .map(|next_region| node.next_point(next_region))
                .unwrap_or(target);
            route.push(RegionRouteStep { region_id, point });
        }
        RegionRouterChangeOutcome::Complete(route)
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), RegionRouterSerializeError> {
        write_count(destination, self.nodes.len(), None)?;
        for node in self.nodes.values() {
            destination.extend_from_slice(&node.region_id.to_le_bytes());
            destination.extend_from_slice(&node.entry.x.to_le_bytes());
            destination.extend_from_slice(&node.entry.y.to_le_bytes());
            destination.extend_from_slice(&node.exit.x.to_le_bytes());
            destination.extend_from_slice(&node.exit.y.to_le_bytes());
            destination.extend_from_slice(&node.exit_range.to_le_bytes());
            write_count(destination, node.next.len(), Some(node.region_id))?;
            for next in node.next.values() {
                destination.extend_from_slice(&next.next_region_id.to_le_bytes());
                destination.extend_from_slice(&next.x.to_le_bytes());
                destination.extend_from_slice(&next.y.to_le_bytes());
            }
        }
        Ok(())
    }
}

impl RegionRouterNode {
    fn next_point(&self, next_region: i32) -> RegionRoutePoint {
        self.next
            .get(&next_region)
            .map(|next| RegionRoutePoint {
                x: next.x,
                y: next.y,
            })
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionRouterSerializeError {
    pub(crate) region_id: Option<i32>,
    pub(crate) count: usize,
}

impl fmt::Display for RegionRouterSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.region_id {
            Some(region_id) => write!(
                formatter,
                "region {region_id} содержит {} переходов вне signed 32-битного диапазона",
                self.count
            ),
            None => write!(
                formatter,
                "RegionRouter содержит {} регионов вне signed 32-битного диапазона",
                self.count
            ),
        }
    }
}

impl Error for RegionRouterSerializeError {}

impl fmt::Display for RegionRouterLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "не удалось прочитать RegionRouter: {error}"),
            Self::MissingToken { field } => {
                write!(formatter, "в RegionRouter отсутствует token поля {field}")
            }
            Self::InvalidInteger { field, token } => write!(
                formatter,
                "в RegionRouter поле {field} содержит нецелое значение {:?}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for RegionRouterLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::MissingToken { .. } | Self::InvalidInteger { .. } => None,
        }
    }
}

struct RegionRouterTokens<'a> {
    tokens: Vec<&'a [u8]>,
    cursor: usize,
}

impl<'a> RegionRouterTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            cursor: 0,
        }
    }

    fn skip(&mut self, field: &'static str) -> Result<(), RegionRouterLoadError> {
        self.next(field).map(|_| ())
    }

    fn read_i32(&mut self, field: &'static str) -> Result<i32, RegionRouterLoadError> {
        self.read_integer(field)
    }

    fn read_u32(&mut self, field: &'static str) -> Result<u32, RegionRouterLoadError> {
        self.read_integer(field)
    }

    fn read_integer<T>(&mut self, field: &'static str) -> Result<T, RegionRouterLoadError>
    where
        T: std::str::FromStr,
    {
        let token = self.next(field)?;
        let value = std::str::from_utf8(token)
            .ok()
            .and_then(|value| value.parse().ok());
        value.ok_or_else(|| RegionRouterLoadError::InvalidInteger {
            field,
            token: token.to_vec(),
        })
    }

    fn next(&mut self, field: &'static str) -> Result<&'a [u8], RegionRouterLoadError> {
        let token = self
            .tokens
            .get(self.cursor)
            .copied()
            .ok_or(RegionRouterLoadError::MissingToken { field })?;
        self.cursor += 1;
        Ok(token)
    }

    fn remaining(&self) -> usize {
        self.tokens.len().saturating_sub(self.cursor)
    }
}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    region_id: Option<i32>,
) -> Result<(), RegionRouterSerializeError> {
    let count_i32 = i32::try_from(count)
        .map_err(|_| RegionRouterSerializeError { region_id, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

// World loader/route search, singleton-а и Game decoder-а.
