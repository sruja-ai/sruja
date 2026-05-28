use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sruja_agent::AgenticMemory;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::MemoryStoreError;

const SCHEMA_VERSION: &str = "1";
const META_FINGERPRINT: &str = "source_fingerprint";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MemorySearchHit {
    pub id: String,
    pub source: String,
    pub trust: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hitl_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MemoryTimelineEntry {
    pub id: String,
    pub source: String,
    pub trust: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hitl_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MemoryTimelineResult {
    pub schema_version: String,
    pub anchor_id: Option<String>,
    pub anchor_timestamp: Option<String>,
    pub entries: Vec<MemoryTimelineEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchMemoryOptions<'a> {
    pub query: &'a str,
    pub element_id: Option<&'a str>,
    pub decision_id: Option<&'a str>,
    pub hitl_kind: Option<&'a str>,
    pub source: Option<&'a str>,
    pub trust: Option<&'a str>,
    pub limit: usize,
}

#[derive(Debug, Clone, Default)]
pub struct TimelineOptions<'a> {
    pub anchor_id: Option<&'a str>,
    pub anchor_timestamp: Option<&'a str>,
    pub before: usize,
    pub after: usize,
    pub decision_id: Option<&'a str>,
    pub element_id: Option<&'a str>,
}

pub struct MemoryStore {
    repo: PathBuf,
    conn: Connection,
}

impl MemoryStore {
    pub fn open(repo: impl AsRef<Path>) -> Result<Self, MemoryStoreError> {
        let repo = repo.as_ref().to_path_buf();
        let db_path = Self::db_path(&repo);
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;",
        )?;
        let mut store = Self { repo, conn };
        store.init_schema()?;
        store.ensure_indexed()?;
        Ok(store)
    }

    pub fn reindex(&mut self) -> Result<(), MemoryStoreError> {
        self.conn.execute_batch(
            "DELETE FROM memory_entries;
             DELETE FROM memory_fts;",
        )?;
        self.index_learnings()?;
        self.index_learned_facts()?;
        self.index_events()?;
        self.index_decisions()?;
        let fp = source_fingerprint(&self.repo)?;
        self.set_meta(META_FINGERPRINT, &fp)?;
        self.set_meta("schema_version", SCHEMA_VERSION)?;
        Ok(())
    }

    pub fn search(
        &self,
        opts: SearchMemoryOptions<'_>,
    ) -> Result<Vec<MemorySearchHit>, MemoryStoreError> {
        let limit = opts.limit.clamp(1, 100);
        let query = opts.query.trim();
        if query.is_empty() {
            return Err(MemoryStoreError::Validation(
                "search query must not be empty".into(),
            ));
        }
        let fts_query = escape_fts_query(query);
        let mut sql = String::from(
            "SELECT e.id, e.source, e.trust, e.timestamp, e.title, e.body,
                    e.element_id, e.decision_id, e.hitl_kind, e.run_id,
                    bm25(memory_fts) AS rank
             FROM memory_fts
             JOIN memory_entries e ON e.id = memory_fts.id
             WHERE memory_fts MATCH ?1",
        );
        let mut bind: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(fts_query)];
        let mut idx = 2;
        if let Some(v) = opts.element_id.filter(|s| !s.is_empty()) {
            sql.push_str(&format!(" AND e.element_id = ?{idx}"));
            bind.push(Box::new(v.to_string()));
            idx += 1;
        }
        if let Some(v) = opts.decision_id.filter(|s| !s.is_empty()) {
            sql.push_str(&format!(" AND e.decision_id = ?{idx}"));
            bind.push(Box::new(v.to_string()));
            idx += 1;
        }
        if let Some(v) = opts.hitl_kind.filter(|s| !s.is_empty()) {
            sql.push_str(&format!(" AND e.hitl_kind = ?{idx}"));
            bind.push(Box::new(v.to_string()));
            idx += 1;
        }
        if let Some(v) = opts.source.filter(|s| !s.is_empty()) {
            sql.push_str(&format!(" AND e.source = ?{idx}"));
            bind.push(Box::new(v.to_string()));
            idx += 1;
        }
        if let Some(v) = opts.trust.filter(|s| !s.is_empty()) {
            sql.push_str(&format!(" AND e.trust = ?{idx}"));
            bind.push(Box::new(v.to_string()));
            idx += 1;
        }
        sql.push_str(&format!(" ORDER BY rank LIMIT ?{idx}"));
        bind.push(Box::new(limit as i64));

        let mut stmt = self.conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::types::ToSql> = bind.iter().map(|b| b.as_ref()).collect();
        let rows = stmt.query_map(params.as_slice(), |row| {
            let body: String = row.get(5)?;
            let snippet = snippet_from_body(&body, query, 240);
            Ok(MemorySearchHit {
                id: row.get(0)?,
                source: row.get(1)?,
                trust: row.get(2)?,
                timestamp: row.get(3)?,
                title: row.get(4)?,
                snippet,
                element_id: row.get(6)?,
                decision_id: row.get(7)?,
                hitl_kind: row.get(8)?,
                run_id: row.get(9)?,
                rank: row.get(10)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn timeline(
        &self,
        opts: TimelineOptions<'_>,
    ) -> Result<MemoryTimelineResult, MemoryStoreError> {
        let anchor_ts = if let Some(id) = opts.anchor_id.filter(|s| !s.is_empty()) {
            self.timestamp_for_id(id)?
        } else if let Some(ts) = opts.anchor_timestamp.filter(|s| !s.is_empty()) {
            parse_timestamp_ms(ts)?
        } else {
            self.latest_timestamp()?
        };

        let before = opts.before.clamp(0, 500);
        let after = opts.after.clamp(0, 500);

        let mut entries = Vec::new();
        entries.extend(self.timeline_window(
            anchor_ts,
            before,
            after,
            opts.decision_id,
            opts.element_id,
        )?);

        Ok(MemoryTimelineResult {
            schema_version: "memory_timeline/v1".to_string(),
            anchor_id: opts.anchor_id.map(str::to_string),
            anchor_timestamp: opts.anchor_timestamp.map(str::to_string),
            entries,
        })
    }

    fn db_path(repo: &Path) -> PathBuf {
        repo.join(".sruja").join("memory.sqlite")
    }

    fn init_schema(&self) -> Result<(), MemoryStoreError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS index_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS memory_entries (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                trust TEXT NOT NULL,
                ts_ms INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                title TEXT,
                body TEXT NOT NULL,
                element_id TEXT,
                decision_id TEXT,
                hitl_kind TEXT,
                run_id TEXT
             );
             CREATE INDEX IF NOT EXISTS idx_memory_ts ON memory_entries(ts_ms);
             CREATE INDEX IF NOT EXISTS idx_memory_decision ON memory_entries(decision_id);
             CREATE INDEX IF NOT EXISTS idx_memory_element ON memory_entries(element_id);
             CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
                id UNINDEXED,
                title,
                body,
                tokenize='porter'
             );",
        )?;
        Ok(())
    }

    fn ensure_indexed(&mut self) -> Result<(), MemoryStoreError> {
        let fp = source_fingerprint(&self.repo)?;
        let stored = self.get_meta(META_FINGERPRINT)?;
        if stored.as_deref() != Some(fp.as_str()) {
            self.reindex()?;
        }
        Ok(())
    }

    fn set_meta(&self, key: &str, value: &str) -> Result<(), MemoryStoreError> {
        self.conn.execute(
            "INSERT INTO index_meta(key, value) VALUES(?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    fn get_meta(&self, key: &str) -> Result<Option<String>, MemoryStoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM index_meta WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(row.get(0)?));
        }
        Ok(None)
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_entry(
        &self,
        id: &str,
        source: &str,
        trust: &str,
        ts_ms: i64,
        timestamp: &str,
        title: Option<&str>,
        body: &str,
        element_id: Option<&str>,
        decision_id: Option<&str>,
        hitl_kind: Option<&str>,
        run_id: Option<&str>,
    ) -> Result<(), MemoryStoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO memory_entries
             (id, source, trust, ts_ms, timestamp, title, body, element_id, decision_id, hitl_kind, run_id)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                id,
                source,
                trust,
                ts_ms,
                timestamp,
                title,
                body,
                element_id,
                decision_id,
                hitl_kind,
                run_id
            ],
        )?;
        self.conn.execute(
            "INSERT OR REPLACE INTO memory_fts(id, title, body) VALUES (?1, ?2, ?3)",
            params![id, title.unwrap_or(""), body],
        )?;
        Ok(())
    }

    fn index_learnings(&self) -> Result<(), MemoryStoreError> {
        let memory = AgenticMemory::load(&self.repo)?;
        for entry in &memory.learnings {
            let ts = entry.timestamp;
            let ts_ms = ts.timestamp_millis();
            let title = entry.hypothesis.chars().take(120).collect::<String>();
            let mut body = format!(
                "context: {}\nhypothesis: {}\nguardrail: {}",
                entry.context, entry.hypothesis, entry.guardrail_advice
            );
            if let Some(reason) = &entry.reason {
                body.push_str("\nreason: ");
                body.push_str(reason);
            }
            let element_id = entry
                .affected_elements
                .first()
                .map(String::as_str)
                .or(entry.selector.as_deref());
            self.insert_entry(
                &entry.id,
                "learning",
                "hypothesis",
                ts_ms,
                &ts.to_rfc3339(),
                Some(&title),
                &body,
                element_id,
                None,
                entry.hitl_kind.as_deref(),
                entry.run_id.as_deref(),
            )?;
        }
        Ok(())
    }

    fn index_events(&self) -> Result<(), MemoryStoreError> {
        let path = self.repo.join(".sruja").join("context_events.jsonl");
        if !path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&path)?;
        for (i, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(line)?;
            let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("event");
            let trust = event_trust(kind);
            let timestamp = v.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
            let ts_ms = parse_timestamp_ms(timestamp).unwrap_or(i as i64);
            let id = v
                .get("run_id")
                .and_then(|r| r.as_str())
                .map(|r| format!("event:{r}:{kind}:{i}"))
                .unwrap_or_else(|| format!("event:{kind}:{i}"));
            let decision_id = v.get("decision_id").and_then(|d| d.as_str());
            let run_id = v.get("run_id").and_then(|r| r.as_str());
            let element_id = v
                .get("elements")
                .and_then(|e| e.as_array())
                .and_then(|a| a.first())
                .and_then(|e| e.as_str())
                .or_else(|| {
                    v.get("subject_ids")
                        .and_then(|e| e.as_array())
                        .and_then(|a| a.first())
                        .and_then(|e| e.as_str())
                });
            let title = Some(kind.to_string());
            let body = serde_json::to_string(&v)?;
            self.insert_entry(
                &id,
                "event",
                trust,
                ts_ms,
                timestamp,
                title.as_deref(),
                &body,
                element_id,
                decision_id,
                None,
                run_id,
            )?;
        }
        Ok(())
    }

    fn index_learned_facts(&self) -> Result<(), MemoryStoreError> {
        #[derive(Debug, Deserialize)]
        struct LearnedFactRow {
            id: String,
            subject: String,
            predicate: String,
            object: String,
            claim: String,
            #[serde(default)]
            confidence: f64,
            #[serde(default)]
            status: String,
            #[serde(default)]
            source: String,
            #[serde(default)]
            evidence_refs: Vec<String>,
        }

        let path = self.repo.join(".sruja").join("learned_facts.jsonl");
        if !path.exists() {
            return Ok(());
        }
        let base_ts_ms = fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or_else(|| Utc::now().timestamp_millis());
        let content = fs::read_to_string(&path)?;
        for (i, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Ok(row) = serde_json::from_str::<LearnedFactRow>(line) else {
                continue;
            };
            let ts_ms = base_ts_ms.saturating_add(i as i64);
            let timestamp = DateTime::<Utc>::from_timestamp_millis(ts_ms)
                .map(|d| d.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339());
            let title = Some(format!("{} {} {}", row.subject, row.predicate, row.object));
            let mut body = format!("claim: {}\nconfidence: {}", row.claim, row.confidence);
            if !row.source.trim().is_empty() {
                body.push_str("\nsource: ");
                body.push_str(row.source.trim());
            }
            if !row.status.trim().is_empty() {
                body.push_str("\nstatus: ");
                body.push_str(row.status.trim());
            }
            if !row.evidence_refs.is_empty() {
                body.push_str("\nevidence_refs:\n");
                for r in &row.evidence_refs {
                    body.push_str("- ");
                    body.push_str(r);
                    body.push('\n');
                }
            }
            let trust = match row.status.to_lowercase().as_str() {
                "reviewed" => "reviewed_truth",
                "rejected" => "hypothesis",
                "stale" => "hypothesis",
                _ => "hypothesis",
            };
            self.insert_entry(
                &format!("learned_fact:{}", row.id),
                "learned_fact",
                trust,
                ts_ms,
                &timestamp,
                title.as_deref(),
                &body,
                Some(row.subject.as_str()),
                None,
                None,
                None,
            )?;
        }
        Ok(())
    }

    fn index_decisions(&self) -> Result<(), MemoryStoreError> {
        let dir = self.repo.join(".sruja").join("decisions");
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("decision")
                .to_string();
            let body = fs::read_to_string(&path)?;
            let elements = parse_elements_from_yaml_frontmatter(&body);
            let title = body
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l.trim_start_matches("# ").to_string())
                .or_else(|| Some(id.clone()));
            let mtime = fs::metadata(&path)?.modified().ok();
            let ts_ms = mtime
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or_else(|| Utc::now().timestamp_millis());
            let timestamp = DateTime::<Utc>::from_timestamp_millis(ts_ms)
                .map(|d| d.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339());
            if elements.is_empty() {
                self.insert_entry(
                    &format!("decision:{id}"),
                    "decision",
                    "reviewed_truth",
                    ts_ms,
                    &timestamp,
                    title.as_deref(),
                    &body,
                    None,
                    Some(id.as_str()),
                    None,
                    None,
                )?;
            } else {
                for el in &elements {
                    self.insert_entry(
                        &format!("decision:{id}:{el}"),
                        "decision",
                        "reviewed_truth",
                        ts_ms,
                        &timestamp,
                        title.as_deref(),
                        &body,
                        Some(el.as_str()),
                        Some(id.as_str()),
                        None,
                        None,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn timestamp_for_id(&self, id: &str) -> Result<i64, MemoryStoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT ts_ms FROM memory_entries WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            return Ok(row.get(0)?);
        }
        Err(MemoryStoreError::Validation(format!(
            "unknown memory id: {id}"
        )))
    }

    fn latest_timestamp(&self) -> Result<i64, MemoryStoreError> {
        let mut stmt = self.conn.prepare("SELECT MAX(ts_ms) FROM memory_entries")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let v: Option<i64> = row.get(0)?;
            if let Some(ts) = v {
                return Ok(ts);
            }
        }
        Ok(Utc::now().timestamp_millis())
    }

    fn timeline_window(
        &self,
        anchor_ts: i64,
        before: usize,
        after: usize,
        decision_id: Option<&str>,
        element_id: Option<&str>,
    ) -> Result<Vec<MemoryTimelineEntry>, MemoryStoreError> {
        let mut sql = String::from(
            "SELECT id, source, trust, timestamp, title, body, element_id, decision_id, hitl_kind
             FROM memory_entries WHERE 1=1",
        );
        let mut bind: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1;
        if let Some(d) = decision_id.filter(|s| !s.is_empty()) {
            sql.push_str(&format!(" AND decision_id = ?{idx}"));
            bind.push(Box::new(d.to_string()));
            idx += 1;
        }
        if let Some(e) = element_id.filter(|s| !s.is_empty()) {
            sql.push_str(&format!(" AND element_id = ?{idx}"));
            bind.push(Box::new(e.to_string()));
            idx += 1;
        }

        let mut prior_sql = sql.clone();
        prior_sql.push_str(&format!(
            " AND ts_ms <= ?{idx} ORDER BY ts_ms DESC LIMIT ?{}",
            idx + 1
        ));
        bind.push(Box::new(anchor_ts));
        bind.push(Box::new(before as i64));

        let mut stmt = self.conn.prepare(&prior_sql)?;
        let params: Vec<&dyn rusqlite::types::ToSql> = bind.iter().map(|b| b.as_ref()).collect();
        let mut prior: Vec<MemoryTimelineEntry> = stmt
            .query_map(params.as_slice(), map_timeline_row)?
            .collect::<Result<Vec<_>, _>>()?;
        prior.reverse();

        let mut after_bind: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut after_idx = 1;
        let mut after_sql = String::from(
            "SELECT id, source, trust, timestamp, title, body, element_id, decision_id, hitl_kind
             FROM memory_entries WHERE ts_ms > ?1",
        );
        after_bind.push(Box::new(anchor_ts));
        after_idx += 1;
        if let Some(d) = decision_id.filter(|s| !s.is_empty()) {
            after_sql.push_str(&format!(" AND decision_id = ?{after_idx}"));
            after_bind.push(Box::new(d.to_string()));
            after_idx += 1;
        }
        if let Some(e) = element_id.filter(|s| !s.is_empty()) {
            after_sql.push_str(&format!(" AND element_id = ?{after_idx}"));
            after_bind.push(Box::new(e.to_string()));
            after_idx += 1;
        }
        after_sql.push_str(&format!(" ORDER BY ts_ms ASC LIMIT ?{after_idx}"));
        after_bind.push(Box::new(after as i64));

        let mut stmt = self.conn.prepare(&after_sql)?;
        let params: Vec<&dyn rusqlite::types::ToSql> =
            after_bind.iter().map(|b| b.as_ref()).collect();
        let mut following: Vec<MemoryTimelineEntry> = stmt
            .query_map(params.as_slice(), map_timeline_row)?
            .collect::<Result<Vec<_>, _>>()?;

        let mut out = prior;
        out.append(&mut following);
        Ok(out)
    }
}

fn map_timeline_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryTimelineEntry> {
    Ok(MemoryTimelineEntry {
        id: row.get(0)?,
        source: row.get(1)?,
        trust: row.get(2)?,
        timestamp: row.get(3)?,
        title: row.get(4)?,
        body: row.get(5)?,
        element_id: row.get(6)?,
        decision_id: row.get(7)?,
        hitl_kind: row.get(8)?,
    })
}

fn event_trust(kind: &str) -> &'static str {
    match kind {
        "decision_accepted" | "decision_applied" | "validation_passed" => "reviewed_truth",
        _ => "hypothesis",
    }
}

fn source_fingerprint(repo: &Path) -> Result<String, MemoryStoreError> {
    use std::time::SystemTime;
    let mut parts = Vec::new();
    for rel in [
        ".sruja/agent_memory.json",
        ".sruja/context_events.jsonl",
        ".sruja/learned_facts.jsonl",
        ".sruja/decisions",
    ] {
        let p = repo.join(rel);
        if p.is_file() {
            let m = fs::metadata(&p)?
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH);
            parts.push(format!("f:{}:{:?}", rel, m));
        } else if p.is_dir() {
            let m = fs::metadata(&p)?
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH);
            parts.push(format!("d:{}:{:?}", rel, m));
            for entry in fs::read_dir(&p)? {
                let entry = entry?;
                let m = entry
                    .metadata()?
                    .modified()
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                parts.push(format!("df:{}:{:?}", entry.path().to_string_lossy(), m));
            }
        }
    }
    Ok(parts.join("|"))
}

fn parse_elements_from_yaml_frontmatter(raw: &str) -> Vec<String> {
    let mut lines = raw.lines().map(str::trim_end);
    let Some(first) = lines.next() else {
        return Vec::new();
    };
    if first.trim() != "---" {
        return Vec::new();
    }

    let mut yaml = String::new();
    for line in lines.by_ref() {
        if line.trim() == "---" {
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
    }

    #[derive(Deserialize)]
    struct Frontmatter {
        #[serde(default)]
        elements: serde_yaml::Value,
    }

    let Ok(fm) = serde_yaml::from_str::<Frontmatter>(&yaml) else {
        return Vec::new();
    };

    let mut out: Vec<String> = Vec::new();
    match fm.elements {
        serde_yaml::Value::Sequence(seq) => {
            for v in seq {
                if let serde_yaml::Value::String(s) = v {
                    if !s.trim().is_empty() {
                        out.push(s.trim().to_string());
                    }
                }
            }
        }
        serde_yaml::Value::String(s) if !s.trim().is_empty() => {
            out.push(s.trim().to_string());
        }
        _ => {}
    }

    out.sort();
    out.dedup();
    out
}

fn parse_timestamp_ms(s: &str) -> Result<i64, MemoryStoreError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.timestamp_millis());
    }
    if let Ok(dt) = chrono::DateTime::parse_from_str(s, "%+") {
        return Ok(dt.timestamp_millis());
    }
    Err(MemoryStoreError::Validation(format!(
        "invalid timestamp: {s}"
    )))
}

fn escape_fts_query(q: &str) -> String {
    q.split_whitespace()
        .map(|w| format!("\"{}\"", w.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn snippet_from_body(body: &str, query: &str, max_len: usize) -> String {
    let lower_body = body.to_lowercase();
    let needle = query
        .split_whitespace()
        .next()
        .unwrap_or(query)
        .to_lowercase();
    let start = lower_body.find(&needle).unwrap_or(0);
    let start = start.saturating_sub(40);
    let end = (start + max_len).min(body.len());
    let mut s = body[start..end].to_string();
    if start > 0 {
        s.insert(0, '…');
    }
    if end < body.len() {
        s.push('…');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_agent::{AgenticMemory, ExperimentOutcome, LearningEntry};
    use tempfile::TempDir;

    fn write_learning_repo(dir: &Path) {
        std::fs::create_dir_all(dir.join(".sruja")).unwrap();
        let mut memory = AgenticMemory::default();
        memory.learnings.push(LearningEntry {
            id: "learn_split".into(),
            kind: None,
            timestamp: Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: "Monolith split".into(),
            hypothesis: "Why did we split the monolith?".into(),
            outcome: ExperimentOutcome::Success,
            reason: None,
            guardrail_advice: "Keep bounded contexts aligned with repo.sruja".into(),
            affected_elements: vec!["Checkout".into()],
            evidence_refs: vec![],
            confidence: None,
            tags: vec![],
            hitl_kind: Some("precedent".into()),
            related_ids: vec![],
            retrieval_count: 0,
            task_success_after: 0,
            task_total_after: 0,
        });
        memory.save(dir).unwrap();
        std::fs::write(
            dir.join(".sruja/learned_facts.jsonl"),
            r#"{"schema_version":"learned_fact/v1","id":"fact_1234","subject":"Checkout","predicate":"depends_on","object":"Payments","claim":"Checkout depends on Payments","evidence_refs":["Cargo.toml"],"confidence":0.8,"status":"observed","source":"learn"}"#
                .to_string()
                + "\n",
        )
        .unwrap();
        std::fs::write(
            dir.join(".sruja/context_events.jsonl"),
            r#"{"schema_version":"context_event/v2","timestamp":"2024-01-02T00:00:00Z","kind":"decision_accepted","outcome":"ok","decision_id":"adr-001","details":{}}"#
                .to_string()
                + "\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.join(".sruja/decisions")).unwrap();
        std::fs::write(
            dir.join(".sruja/decisions/adr-001.md"),
            "---\nid: adr-001\ntype: adr\nstatus: accepted\nscope: repo\nelements:\n  - Checkout\n  - Payments\n---\n# Why we split the monolith\n\nAccepted bounded context split.\n",
        )
        .unwrap();
    }

    #[test]
    fn search_finds_learning_and_decision() {
        let tmp = TempDir::new().unwrap();
        write_learning_repo(tmp.path());
        let store = MemoryStore::open(tmp.path()).unwrap();
        let hits = store
            .search(SearchMemoryOptions {
                query: "monolith",
                limit: 10,
                ..Default::default()
            })
            .unwrap();
        assert!(!hits.is_empty());
        assert!(hits
            .iter()
            .any(|h| h.trust == "hypothesis" || h.trust == "reviewed_truth"));
    }

    #[test]
    fn search_can_filter_by_element_id_for_decisions_and_learned_facts() {
        let tmp = TempDir::new().unwrap();
        write_learning_repo(tmp.path());
        let store = MemoryStore::open(tmp.path()).unwrap();

        let decision_hits = store
            .search(SearchMemoryOptions {
                query: "split",
                element_id: Some("Checkout"),
                limit: 10,
                ..Default::default()
            })
            .unwrap();
        assert!(decision_hits.iter().any(|h| h.source == "decision"));

        let fact_hits = store
            .search(SearchMemoryOptions {
                query: "depends",
                element_id: Some("Checkout"),
                limit: 10,
                ..Default::default()
            })
            .unwrap();
        assert!(fact_hits.iter().any(|h| h.source == "learned_fact"));
    }

    #[test]
    fn timeline_returns_entries_around_anchor() {
        let tmp = TempDir::new().unwrap();
        write_learning_repo(tmp.path());
        let store = MemoryStore::open(tmp.path()).unwrap();
        let tl = store
            .timeline(TimelineOptions {
                anchor_id: Some("learn_split"),
                before: 5,
                after: 5,
                ..Default::default()
            })
            .unwrap();
        assert!(!tl.entries.is_empty());
    }
}
