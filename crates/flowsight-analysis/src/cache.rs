//! 执行流分析缓存
//!
//! 提供多级缓存以优化重复分析：
//! - 文件解析结果缓存 (基于内容 hash)
//! - 执行流缓存 (基于文件 + 入口函数)
//! - 入口点检测缓存

use flowsight_core::{FlowNode, Result};
use flowsight_parser::ParseResult;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 缓存条目
#[derive(Clone)]
struct CacheEntry<T> {
    /// 缓存的值
    value: T,
    /// 创建时间
    created_at: Instant,
    /// 访问次数
    _access_count: u64,
}

impl<T> CacheEntry<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            _access_count: 1,
        }
    }

    fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// 内容哈希计算
fn content_hash(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 解析结果最大缓存数
    pub max_parse_entries: usize,
    /// 执行流最大缓存数
    pub max_flow_entries: usize,
    /// 缓存过期时间 (秒)
    pub ttl_seconds: u64,
    /// 启用缓存统计
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_parse_entries: 100,
            max_flow_entries: 500,
            ttl_seconds: 300, // 5 分钟
            enable_stats: true,
        }
    }
}

/// 缓存统计
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// 解析缓存命中
    pub parse_hits: u64,
    /// 解析缓存未命中
    pub parse_misses: u64,
    /// 执行流缓存命中
    pub flow_hits: u64,
    /// 执行流缓存未命中
    pub flow_misses: u64,
    /// 入口点缓存命中
    pub entry_hits: u64,
    /// 入口点缓存未命中
    pub entry_misses: u64,
}

impl CacheStats {
    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        let total_hits = self.parse_hits + self.flow_hits + self.entry_hits;
        let total_misses = self.parse_misses + self.flow_misses + self.entry_misses;
        let total = total_hits + total_misses;
        if total == 0 {
            0.0
        } else {
            total_hits as f64 / total as f64
        }
    }
}

/// 执行流分析缓存
pub struct AnalysisCache {
    /// 配置
    config: CacheConfig,
    /// 解析结果缓存 (content_hash -> ParseResult)
    parse_cache: RwLock<HashMap<u64, CacheEntry<Arc<ParseResult>>>>,
    /// 执行流缓存 ((content_hash, entry_function) -> FlowNode)
    flow_cache: RwLock<HashMap<(u64, String), CacheEntry<Arc<FlowNode>>>>,
    /// 入口点缓存 (content_hash -> entry_points)
    entry_cache: RwLock<HashMap<u64, CacheEntry<Arc<Vec<String>>>>>,
    /// 缓存统计
    stats: RwLock<CacheStats>,
}

impl AnalysisCache {
    /// 创建新的缓存实例
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// 使用自定义配置创建缓存
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            config,
            parse_cache: RwLock::new(HashMap::new()),
            flow_cache: RwLock::new(HashMap::new()),
            entry_cache: RwLock::new(HashMap::new()),
            stats: RwLock::new(CacheStats::default()),
        }
    }

    /// 获取或计算解析结果
    pub fn get_or_parse<F>(&self, content: &str, parse_fn: F) -> Result<Arc<ParseResult>>
    where
        F: FnOnce(&str) -> Result<ParseResult>,
    {
        let hash = content_hash(content);

        // 尝试从缓存获取
        {
            let cache = self.parse_cache.read().unwrap();
            if let Some(entry) = cache.get(&hash) {
                if entry.age() < Duration::from_secs(self.config.ttl_seconds) {
                    if self.config.enable_stats {
                        self.stats.write().unwrap().parse_hits += 1;
                    }
                    return Ok(Arc::clone(&entry.value));
                }
            }
        }

        // 缓存未命中，执行解析
        if self.config.enable_stats {
            self.stats.write().unwrap().parse_misses += 1;
        }

        let result = parse_fn(content)?;
        let arc_result = Arc::new(result);

        // 存入缓存
        {
            let mut cache = self.parse_cache.write().unwrap();

            // 检查是否需要清理
            if cache.len() >= self.config.max_parse_entries {
                self.evict_oldest(&mut cache);
            }

            cache.insert(hash, CacheEntry::new(Arc::clone(&arc_result)));
        }

        Ok(arc_result)
    }

    /// 获取或构建执行流
    pub fn get_or_build_flow<F>(
        &self,
        content: &str,
        entry_function: &str,
        build_fn: F,
    ) -> Option<Arc<FlowNode>>
    where
        F: FnOnce() -> Option<FlowNode>,
    {
        let hash = content_hash(content);
        let key = (hash, entry_function.to_string());

        // 尝试从缓存获取
        {
            let cache = self.flow_cache.read().unwrap();
            if let Some(entry) = cache.get(&key) {
                if entry.age() < Duration::from_secs(self.config.ttl_seconds) {
                    if self.config.enable_stats {
                        self.stats.write().unwrap().flow_hits += 1;
                    }
                    return Some(Arc::clone(&entry.value));
                }
            }
        }

        // 缓存未命中，执行构建
        if self.config.enable_stats {
            self.stats.write().unwrap().flow_misses += 1;
        }

        let result = build_fn()?;
        let arc_result = Arc::new(result);

        // 存入缓存
        {
            let mut cache = self.flow_cache.write().unwrap();

            // 检查是否需要清理
            if cache.len() >= self.config.max_flow_entries {
                self.evict_oldest_flow(&mut cache);
            }

            cache.insert(key, CacheEntry::new(Arc::clone(&arc_result)));
        }

        Some(arc_result)
    }

    /// 获取或检测入口点
    pub fn get_or_detect_entries<F>(&self, content: &str, detect_fn: F) -> Arc<Vec<String>>
    where
        F: FnOnce() -> Vec<String>,
    {
        let hash = content_hash(content);

        // 尝试从缓存获取
        {
            let cache = self.entry_cache.read().unwrap();
            if let Some(entry) = cache.get(&hash) {
                if entry.age() < Duration::from_secs(self.config.ttl_seconds) {
                    if self.config.enable_stats {
                        self.stats.write().unwrap().entry_hits += 1;
                    }
                    return Arc::clone(&entry.value);
                }
            }
        }

        // 缓存未命中，执行检测
        if self.config.enable_stats {
            self.stats.write().unwrap().entry_misses += 1;
        }

        let result = detect_fn();
        let arc_result = Arc::new(result);

        // 存入缓存
        {
            let mut cache = self.entry_cache.write().unwrap();
            cache.insert(hash, CacheEntry::new(Arc::clone(&arc_result)));
        }

        arc_result
    }

    /// 清除所有缓存
    pub fn clear(&self) {
        self.parse_cache.write().unwrap().clear();
        self.flow_cache.write().unwrap().clear();
        self.entry_cache.write().unwrap().clear();
    }

    /// 清除特定文件的缓存
    pub fn invalidate(&self, content: &str) {
        let hash = content_hash(content);

        self.parse_cache.write().unwrap().remove(&hash);
        self.entry_cache.write().unwrap().remove(&hash);

        // 清除该文件的所有执行流缓存
        let mut flow_cache = self.flow_cache.write().unwrap();
        flow_cache.retain(|(h, _), _| *h != hash);
    }

    /// 获取缓存统计
    pub fn stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }

    /// 获取缓存大小
    pub fn size(&self) -> (usize, usize, usize) {
        (
            self.parse_cache.read().unwrap().len(),
            self.flow_cache.read().unwrap().len(),
            self.entry_cache.read().unwrap().len(),
        )
    }

    /// 清理过期缓存
    pub fn cleanup_expired(&self) {
        let ttl = Duration::from_secs(self.config.ttl_seconds);

        {
            let mut cache = self.parse_cache.write().unwrap();
            cache.retain(|_, entry| entry.age() < ttl);
        }

        {
            let mut cache = self.flow_cache.write().unwrap();
            cache.retain(|_, entry| entry.age() < ttl);
        }

        {
            let mut cache = self.entry_cache.write().unwrap();
            cache.retain(|_, entry| entry.age() < ttl);
        }
    }

    /// 清理最旧的解析缓存条目
    fn evict_oldest<T>(&self, cache: &mut HashMap<u64, CacheEntry<T>>) {
        // 找到最旧的条目
        if let Some((&oldest_key, _)) = cache.iter().min_by_key(|(_, entry)| entry.created_at) {
            cache.remove(&oldest_key);
        }
    }

    /// 清理最旧的执行流缓存条目
    fn evict_oldest_flow<T>(&self, cache: &mut HashMap<(u64, String), CacheEntry<T>>) {
        // 找到最旧的条目
        if let Some((oldest_key, _)) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(k, _)| (k.clone(), ()))
        {
            cache.remove(&oldest_key);
        }
    }
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局缓存实例
static GLOBAL_CACHE: std::sync::OnceLock<AnalysisCache> = std::sync::OnceLock::new();

/// 获取全局缓存实例
pub fn global_cache() -> &'static AnalysisCache {
    GLOBAL_CACHE.get_or_init(AnalysisCache::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit_miss() {
        let cache = AnalysisCache::new();
        let content = "int main() { return 0; }";

        // 第一次应该是 miss
        let entries1 = cache.get_or_detect_entries(content, || vec!["main".to_string()]);
        assert_eq!(*entries1, vec!["main".to_string()]);

        // 第二次应该是 hit
        let entries2 = cache.get_or_detect_entries(content, || {
            panic!("Should not be called on cache hit");
        });
        assert_eq!(*entries2, vec!["main".to_string()]);

        // 检查统计
        let stats = cache.stats();
        assert_eq!(stats.entry_hits, 1);
        assert_eq!(stats.entry_misses, 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = AnalysisCache::new();
        let content = "int foo() { return 1; }";

        // 填充缓存
        let _ = cache.get_or_detect_entries(content, || vec!["foo".to_string()]);

        // 确认缓存有数据
        let (_, _, entry_count) = cache.size();
        assert_eq!(entry_count, 1);

        // 清除缓存
        cache.invalidate(content);

        // 确认缓存为空
        let (_, _, entry_count) = cache.size();
        assert_eq!(entry_count, 0);
    }

    #[test]
    fn test_different_content_different_cache() {
        let cache = AnalysisCache::new();

        let content1 = "int foo() { return 1; }";
        let content2 = "int bar() { return 2; }";

        let entries1 = cache.get_or_detect_entries(content1, || vec!["foo".to_string()]);
        let entries2 = cache.get_or_detect_entries(content2, || vec!["bar".to_string()]);

        assert_eq!(*entries1, vec!["foo".to_string()]);
        assert_eq!(*entries2, vec!["bar".to_string()]);

        // 两个不同的缓存条目
        let (_, _, entry_count) = cache.size();
        assert_eq!(entry_count, 2);
    }

    #[test]
    fn test_hit_rate() {
        let cache = AnalysisCache::new();
        let content = "test content";

        // 1 miss, 3 hits
        for _ in 0..4 {
            cache.get_or_detect_entries(content, || vec![]);
        }

        let stats = cache.stats();
        assert_eq!(stats.entry_misses, 1);
        assert_eq!(stats.entry_hits, 3);
        // hit_rate = 3 / 4 = 0.75
        assert!((stats.hit_rate() - 0.75).abs() < 0.001);
    }
}
