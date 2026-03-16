use serde::{Deserialize, Serialize};

/// 单条分数记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    /// 排名 (1-10)
    pub rank: usize,
    /// 分数
    pub score: i32,
    /// 步数
    pub moves: u32,
    /// 日期 "2024-01-15"
    pub date: String,
}

/// 排行榜（单个难度）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Leaderboard {
    /// 最多 10 条记录，按分数降序排列
    pub entries: Vec<ScoreEntry>,
}

impl Leaderboard {
    /// 尝试插入新记录
    /// 返回排名（1-based）如果进入前十，否则返回 None
    pub fn try_insert(&mut self, score: i32, moves: u32, date: &str) -> Option<usize> {
        // 找到插入位置（分数降序）
        let insert_pos = self.entries.iter().position(|e| score > e.score).unwrap_or(self.entries.len());

        // 如果列表已满且不在前十，返回 None
        if insert_pos >= 10 {
            return None;
        }

        // 插入新记录
        let new_entry = ScoreEntry {
            rank: insert_pos + 1,
            score,
            moves,
            date: date.to_string(),
        };
        self.entries.insert(insert_pos, new_entry);

        // 保留前 10 条
        if self.entries.len() > 10 {
            self.entries.truncate(10);
        }

        // 更新所有排名
        for (i, entry) in self.entries.iter_mut().enumerate() {
            entry.rank = i + 1;
        }

        Some(insert_pos + 1)
    }
}

/// 游戏统计（全局）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameStats {
    /// 三个难度的排行榜（索引 0-2 对应难度 1-3）
    pub leaderboards: [Leaderboard; 3],
    /// 各难度游戏场次
    pub games_played: [u32; 3],
    /// 各难度胜利场次
    pub games_won: [u32; 3],
}

impl GameStats {
    /// 创建新的统计数据
    pub fn new() -> Self {
        Self {
            leaderboards: Default::default(),
            games_played: [0; 3],
            games_won: [0; 3],
        }
    }

    /// 记录一局游戏结果
    /// difficulty: 1-3
    /// 返回 (更新后统计, 排名或None)
    pub fn record_game(&mut self, difficulty: u8, score: i32, moves: u32, won: bool, date: &str) -> Option<usize> {
        let idx = (difficulty - 1) as usize;

        // 更新游戏统计
        self.games_played[idx] += 1;
        if won {
            self.games_won[idx] += 1;
        }

        // 只有胜利才计入排行榜
        if won {
            self.leaderboards[idx].try_insert(score, moves, date)
        } else {
            None
        }
    }
}
