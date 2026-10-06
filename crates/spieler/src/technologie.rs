//! One rule-derived graph for the native and browser clients.
use kern::{Einheit, Forschung, Gebaeude};
use serde_json::{json, Value};
use crate::Session;

pub fn baum(session: &Session) -> Value {
    let v=session.view(); let r=session.regeln();
    let p=v["planeten"].as_array().and_then(|a|a.iter().find(|p|p["heimat"]==true)).unwrap_or(&Value::Null);
    let stage=v["stufe"].as_u64().unwrap_or(1);
    let mut nodes=Vec::new();
    for g in Gebaeude::ALLE {
        let rule=r.geb(g);
        let deps:Vec<Value>=rule.braucht.iter().map(|(b,n)|json!({"id":format!("buildings.{}",b.name()),"level":n,"have":p["gebaeude"][b.name()].as_u64().unwrap_or(0)})).collect();
        nodes.push(json!({"id":format!("buildings.{}",g.name()),"key":g.name(),"category":"buildings","stage":rule.ab_stufe,
            "level":p["gebaeude"][g.name()].as_u64().unwrap_or(0),"effect":rule.wirkung,"deps":deps,"action":{"typ":"bauen","planet":p["koord"],"gebaeude":g.name()}}));
    }
    for f in Forschung::ALLE {
        let rule=r.forsch(f);
        nodes.push(json!({"id":format!("research.{}",f.name()),"key":f.name(),"category":"research","stage":rule.ab_stufe,
            "level":v["forschung"]["stufen"][f.name()].as_u64().unwrap_or(0),"effect":rule.wirkung,
            "active":v["forschung"]["aktiv"]["forschung"]==f.name(),
            "deps":[{"id":"buildings.labor","level":rule.labor,"have":p["gebaeude"]["labor"].as_u64().unwrap_or(0)}],
            "action":{"typ":"forschen","forschung":f.name()}}));
    }
    for e in Einheit::ALLE {
        let rule=r.einh(e);let category=if e.ist_schiff(){"ships"}else{"defenses"};
        let mut deps:Vec<Value>=rule.braucht.iter().map(|(b,n)|json!({"id":format!("buildings.{}",b.name()),"level":n,"have":p["gebaeude"][b.name()].as_u64().unwrap_or(0)})).collect();
        if e.ist_schiff(){deps.push(json!({"id":"buildings.werft","level":rule.werft.max(1),"have":p["gebaeude"]["werft"].as_u64().unwrap_or(0)}));}
        nodes.push(json!({"id":format!("{category}.{}",e.name()),"key":e.name(),"category":category,"stage":rule.ab_stufe,
            "level":p[if e.ist_schiff(){"schiffe"}else{"verteidigung"}][e.name()].as_u64().unwrap_or(0),"effect":rule.wirkung,"deps":deps,
            "bonus":rule.antrieb.map(|f|f.name()),"action":{"typ":"fertigen","planet":p["koord"],"einheit":e.name(),"anzahl":1}}));
    }
    for n in &mut nodes {
        let accessible=n["stage"].as_u64().unwrap()<=stage && n["deps"].as_array().unwrap().iter().all(|d|d["have"].as_u64()>=d["level"].as_u64());
        n["accessible"]=json!(accessible);
        let mut q=json!({"typ":"kosten","planet":p["koord"]});
        q[match n["category"].as_str().unwrap(){"buildings"=>"gebaeude","research"=>"forschung",_=>"einheit"}]=n["key"].clone();
        n["quote"]=session.query(&q).unwrap_or(Value::Null);
        n["affordable"]=json!(n["quote"]["kosten"].as_object().map(|c|c.iter().all(|(k,a)|p["bestand"][k].as_i64().unwrap_or(0)>=a.as_i64().unwrap_or(i64::MAX))).unwrap_or(false));
    }
    json!({"stage":stage,"nodes":nodes,"stages":["Gründung","Industrie","Orbit","Sternenflug","Imperium"]})
}
