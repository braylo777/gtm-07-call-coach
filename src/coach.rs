//! Call coaching metrics. Rep vs prospect talk share by word count, rep questions, longest rep
//! monologue (consecutive rep words in seconds when timestamps exist), topic hits and next steps.
use crate::text::words;
use crate::util::*;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

const NEXT: &[&str] = &["next step", "i'll send", "i will send", "let's schedule", "follow up", "send over", "book", "loop in", "we'll get", "by friday", "by monday"];

pub fn coach_op(i: &Value) -> R<Value> {
    let topics = obj(i, "topics")?;
    let tg = i.get("targets").cloned().unwrap_or(json!({}));
    let max_talk = f_or(&tg, "max_rep_talk_pct", 60.0);
    let min_q = f_or(&tg, "min_rep_questions", 6.0);
    let max_mono = f_or(&tg, "max_monologue_sec", 90.0);
    let mut per = vec![];
    let mut trend: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for c in arr(i, "calls")? {
        let id = s(c, "call_id")?;
        let rep = s(c, "rep")?;
        let u = arr(c, "utterances")?;
        let (mut rw, mut pw, mut q) = (0usize, 0usize, 0usize);
        let (mut mono, mut cur_start): (f64, Option<f64>) = (0.0, None);
        let mut hits: Map<String, Value> = Map::new();
        let mut steps = vec![];
        for (k, x) in u.iter().enumerate() {
            let text = s_or(x, "text", "");
            let is_rep = s_or(x, "role", "") == "rep";
            let t = f_or(x, "t", k as f64 * 15.0);
            if is_rep {
                rw += words(text);
                q += text.matches('?').count();
                if cur_start.is_none() {
                    cur_start = Some(t);
                }
                let end = u.get(k + 1).map(|n| f_or(n, "t", t + 15.0)).unwrap_or(t + 15.0);
                mono = mono.max(end - cur_start.unwrap());
            } else {
                pw += words(text);
                cur_start = None;
            }
            let low = text.to_lowercase();
            for (topic, kws) in topics {
                let kws = kws.as_array().ok_or("topic keywords must be arrays")?;
                if kws.iter().filter_map(Value::as_str).any(|kw| low.contains(&kw.to_lowercase())) {
                    let e = hits.entry(topic.clone()).or_insert(json!([]));
                    e.as_array_mut().unwrap().push(json!(t));
                }
            }
            if NEXT.iter().any(|p| low.contains(p)) {
                steps.push(json!({"t_sec": t, "speaker": s_or(x, "speaker", "?"), "text": text}));
            }
        }
        let talk = rw as f64 / (rw + pw).max(1) as f64 * 100.0;
        let mut flags = vec![];
        if talk > max_talk {
            flags.push(format!("rep talked {:.0}% (target ≤ {max_talk:.0}%)", talk));
        }
        if (q as f64) < min_q {
            flags.push(format!("{q} rep questions (target ≥ {min_q:.0})"));
        }
        if mono > max_mono {
            flags.push(format!("longest monologue {mono:.0}s (target ≤ {max_mono:.0}s)"));
        }
        if steps.is_empty() {
            flags.push("no next step agreed".into());
        }
        trend.entry(rep.into()).or_default().push((talk, q as f64));
        per.push(json!({"call_id": id, "rep": rep, "rep_talk_pct": rd(talk, 1), "rep_questions": q, "longest_monologue_sec": mono,
            "topics": hits, "next_steps": steps, "coaching_flags": flags, "score": 100 - 25 * flags.len().min(4)}));
    }
    let reps: Map<String, Value> = trend.into_iter().map(|(r, v)| {
        let first = v.first().unwrap();
        let last = v.last().unwrap();
        (r, json!({"calls": v.len(), "talk_pct_first_to_last": [rd(first.0, 1), rd(last.0, 1)], "questions_first_to_last": [first.1, last.1],
            "improving": v.len() > 1 && last.0 <= first.0 && last.1 >= first.1}))
    }).collect();
    Ok(json!({"calls": per, "reps": reps}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn metrics_and_flags() {
        let i = json!({"topics":{"pricing":["price","cost"]},"targets":{"min_rep_questions":1},"calls":[{"call_id":"c","rep":"r","utterances":[
            {"role":"rep","t":0,"text":"one two three four"},{"role":"rep","t":100,"text":"five six?"},{"role":"prospect","t":200,"text":"What's the price? I'll send the deck."}]}]});
        let r = coach_op(&i).unwrap();
        let c = &r["calls"][0];
        assert_eq!(c["rep_talk_pct"], 46.2); // 6 rep words of 13
        assert_eq!(c["rep_questions"], 1);
        assert_eq!(c["longest_monologue_sec"], 200.0);
        assert_eq!(c["topics"]["pricing"], json!([200.0]));
        assert_eq!(c["coaching_flags"].as_array().unwrap().len(), 1);
    }
}
