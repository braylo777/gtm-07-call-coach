//! Battlecards. For each competitor, count calls and mentions, find objection categories said
//! in the same or the next prospect utterance, and attach playbook responses and proof points.
use crate::util::*;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub fn cards_op(i: &Value) -> R<Value> {
    let comps = obj(i, "competitors")?; // name -> aliases
    let objections = obj(i, "objections")?; // category -> keywords
    let playbook = i.get("playbook").and_then(Value::as_object).cloned().unwrap_or_default();
    let mut cards: BTreeMap<String, (BTreeSet<String>, usize, BTreeMap<String, usize>, Vec<Value>, usize)> = BTreeMap::new();
    for c in arr(i, "calls")? {
        let id = s(c, "call_id")?;
        let won = c.get("outcome").and_then(Value::as_str);
        let u = arr(c, "utterances")?;
        let mut in_call = BTreeSet::new();
        for (k, x) in u.iter().enumerate() {
            let low = s_or(x, "text", "").to_lowercase();
            for (name, al) in comps {
                let mut names = vec![name.to_lowercase()];
                names.extend(al.as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_lowercase));
                if !names.iter().any(|n| low.contains(n.as_str())) {
                    continue;
                }
                let e = cards.entry(name.clone()).or_default();
                e.0.insert(id.to_string());
                e.1 += 1;
                in_call.insert(name.clone());
                let ctx = format!("{} {}", low, u.get(k + 1).map(|n| s_or(n, "text", "").to_lowercase()).unwrap_or_default());
                for (cat, kws) in objections {
                    if kws.as_array().into_iter().flatten().filter_map(Value::as_str).any(|w| ctx.contains(&w.to_lowercase())) {
                        *e.2.entry(cat.clone()).or_default() += 1;
                    }
                }
                if e.3.len() < 3 {
                    e.3.push(json!({"call_id": id, "t_sec": x.get("t"), "quote": s_or(x, "text", "")}));
                }
            }
        }
        if won == Some("won") {
            for n in in_call {
                cards.get_mut(&n).unwrap().4 += 1;
            }
        }
    }
    let mut out = vec![];
    for (name, (calls, mentions, objs, quotes, wins)) in cards {
        let mut top: Vec<(String, usize)> = objs.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let responses: Vec<Value> = top.iter().map(|(cat, n)| json!({"objection": cat, "times": n,
            "say": playbook.get(cat).and_then(|p| p.get("response")).cloned().unwrap_or(json!("(no playbook entry: write one)")),
            "proof": playbook.get(cat).and_then(|p| p.get("proof")).cloned().unwrap_or(Value::Null)})).collect();
        out.push(json!({"competitor": name, "calls": calls.len(), "mentions": mentions, "win_rate_when_mentioned_pct": rd(wins as f64 / calls.len() as f64 * 100.0, 1),
            "top_objections": responses, "verbatims": quotes}));
    }
    out.sort_by(|a, b| b["calls"].as_u64().cmp(&a["calls"].as_u64()));
    Ok(json!({"battlecards": out}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn card_from_mentions() {
        let i = json!({"competitors":{"Acme":["acme.io"]},"objections":{"price":["cheaper"]},"playbook":{"price":{"response":"TCO"}},
            "calls":[{"call_id":"1","outcome":"won","utterances":[{"text":"We also use Acme."},{"text":"They are cheaper."}]},{"call_id":"2","outcome":"lost","utterances":[{"text":"acme.io trial"}]}]});
        let r = cards_op(&i).unwrap();
        let c = &r["battlecards"][0];
        assert_eq!((c["calls"].as_u64(), c["mentions"].as_u64()), (Some(2), Some(2)));
        assert_eq!(c["win_rate_when_mentioned_pct"], 50.0);
        assert_eq!(c["top_objections"][0]["say"], "TCO");
    }
}
