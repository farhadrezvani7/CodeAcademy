//! Ranking among students of the same level (by XP).

use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Contender {
    pub user_id: i32,
    pub name: String,
    pub xp: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ranked {
    pub rank: usize,
    pub user_id: i32,
    pub name: String,
    pub xp: i32,
}

/// Standard competition ranking ("1224"): equal XP shares a rank.
/// Ties are listed by user id so ordering is stable.
pub fn rank(mut contenders: Vec<Contender>) -> Vec<Ranked> {
    contenders.sort_by(|a, b| b.xp.cmp(&a.xp).then(a.user_id.cmp(&b.user_id)));
    let mut out: Vec<Ranked> = Vec::with_capacity(contenders.len());
    for (i, c) in contenders.into_iter().enumerate() {
        let rank = match out.last() {
            Some(prev) if prev.xp == c.xp => prev.rank,
            _ => i + 1,
        };
        out.push(Ranked { rank, user_id: c.user_id, name: c.name, xp: c.xp });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(id: i32, xp: i32) -> Contender {
        Contender { user_id: id, name: format!("u{id}"), xp }
    }

    #[test]
    fn orders_by_xp_with_shared_ranks() {
        let ranked = rank(vec![c(1, 100), c(2, 300), c(3, 100), c(4, 50)]);
        let pairs: Vec<(i32, usize)> = ranked.iter().map(|r| (r.user_id, r.rank)).collect();
        assert_eq!(pairs, vec![(2, 1), (1, 2), (3, 2), (4, 4)]);
    }

    #[test]
    fn empty() {
        assert!(rank(vec![]).is_empty());
    }
}
