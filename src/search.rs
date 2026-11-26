//! TF-IDF vector search. Each document is a window of `window` consecutive utterances, so a
//! hit carries a little context. Vectors are L2-normalized; score = cosine similarity.
use crate::text::tokens;
use crate::util::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub struct Doc {
    pub call: String,
    pub t: f64,
    pub speaker: String,
    pub text: String,
}

pub fn build_docs(calls: &[Value], window: usize) -> R<Vec<Doc>> {
    let mut docs = vec![];
    for c in calls {
        let id = s(c, "call_id")?;
        let u = arr(c, "utterances")?;
        for k in 0..u.len() {
            let end = (k + window).min(u.len());
            let text = u[k..end].iter().map(|x| format!("{}: {}", s_or(x, "speaker", "?"), s_or(x, "text", ""))).collect::<Vec<_>>().join(" | ");
            docs.push(Doc { call: id.into(), t: f_or(&u[k], "t", 0.0), speaker: s_or(&u[k], "speaker", "?").into(), text });
        }
    }
    Ok(docs)
}

fn vectorize(tok: &[String], idf: &BTreeMap<String, f64>) -> BTreeMap<String, f64> {
    let mut tf: BTreeMap<String, f64> = BTreeMap::new();
    for w in tok {
        *tf.entry(w.clone()).or_default() += 1.0;
    }
    let mut v: BTreeMap<String, f64> = tf.into_iter().map(|(w, c)| { let x = (1.0 + f64::ln(c)) * idf.get(&w).copied().unwrap_or(0.0); (w, x) }).collect();
    let n = v.values().map(|x| x * x).sum::<f64>().sqrt();
    if n > 0.0 {
        v.values_mut().for_each(|x| *x /= n);
    }
    v
}

pub fn search_op(i: &Value) -> R<Value> {
    let q = s(i, "query")?;
    let k = f_or(i, "top_k", 3.0) as usize;
    let window = (f_or(i, "window", 2.0) as usize).max(1);
    let docs = build_docs(arr(i, "calls")?, window)?;
    if docs.is_empty() {
        return Err("no utterances to search".into());
    }
    let toks: Vec<Vec<String>> = docs.iter().map(|d| tokens(&d.text)).collect();
    let mut df: BTreeMap<String, f64> = BTreeMap::new();
    for t in &toks {
        let mut u = t.clone();
        u.sort();
        u.dedup();
        for w in u {
            *df.entry(w).or_default() += 1.0;
        }
    }
    let n = docs.len() as f64;
    let idf: BTreeMap<String, f64> = df.into_iter().map(|(w, d)| (w, f64::ln((1.0 + n) / (1.0 + d)) + 1.0)).collect();
    let qv = vectorize(&tokens(q), &idf);
    if qv.is_empty() {
        return Err("query has no searchable words".into());
    }
    let mut hits: Vec<(f64, usize)> = toks.iter().enumerate().map(|(j, t)| {
        let dv = vectorize(t, &idf);
        (qv.iter().map(|(w, x)| x * dv.get(w).copied().unwrap_or(0.0)).sum::<f64>(), j)
    }).filter(|h| h.0 > 0.0).collect();
    hits.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    // one hit per (call, start) neighbourhood: skip windows overlapping a better hit
    let mut taken: Vec<usize> = vec![];
    let mut out = vec![];
    for (sc, j) in hits {
        if taken.iter().any(|&t| docs[t].call == docs[j].call && (t as i64 - j as i64).unsigned_abs() < window as u64) {
            continue;
        }
        taken.push(j);
        let d = &docs[j];
        out.push(json!({"score": rd(sc, 3), "call_id": d.call, "t_sec": d.t, "speaker": d.speaker, "snippet": d.text}));
        if out.len() >= k {
            break;
        }
    }
    Ok(json!({"query": q, "documents_indexed": docs.len(), "vocabulary": idf.len(), "hits": out}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn finds_relevant_window() {
        let i = json!({"query":"SOC 2 security review","window":1,"calls":[{"call_id":"c1","utterances":[
            {"speaker":"A","text":"Let's talk timeline."},{"speaker":"B","text":"Security needs your SOC 2 report before review."},{"speaker":"A","text":"Pricing is per seat."}]}]});
        let r = search_op(&i).unwrap();
        assert_eq!(r["hits"][0]["speaker"], "B");
        assert_eq!(r["hits"].as_array().unwrap().len(), 1);
        assert!(search_op(&json!({"query":"the and","calls":i["calls"]})).is_err());
    }
}
