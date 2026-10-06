use crate::Db;
use base64::{engine::general_purpose::STANDARD,Engine};
use kern::{Regelwerk,Gebaeude,Forschung,Einheit,Gut,Rolle};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,fs,path::{Path,PathBuf},time::{SystemTime,UNIX_EPOCH}};

fn hash(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
fn read(path:&Path)->Result<String,String>{fs::read_to_string(path).map_err(|e|format!("{}: {e}",path.display()))}
fn json_file(path:&Path)->Result<Value,String>{serde_json::from_str(&read(path)?).map_err(|e|format!("{}: {e}",path.display()))}
fn insert(db:&Db,table:&str,values:&[String])->Result<(),String>{
    let sql=format!("INSERT INTO {table} VALUES ({})",vec!["?";values.len()].join(","));
    let refs:Vec<&str>=values.iter().map(String::as_str).collect(); db.exec(&sql,&refs)
}
fn entity(db:&Db,id:&str,kind:&str,title:&str,status:&str,faction:&str,source:&str,line:usize,data:&Value,body:&str)->Result<(),String>{
    insert(db,"entities",&[id.into(),kind.into(),title.into(),status.into(),faction.into(),source.into(),line.to_string(),data.to_string(),body.into()])
}
fn relation(db:&Db,from:&str,kind:&str,to:&str,details:Value)->Result<(),String>{insert(db,"relations",&[from.into(),kind.into(),to.into(),details.to_string()])}
fn collect(root:&Path,dir:&Path,out:&mut Vec<PathBuf>)->Result<(),String>{
    for entry in fs::read_dir(dir).map_err(|e|e.to_string())?{
        let p=entry.map_err(|e|e.to_string())?.path(); let meta=fs::symlink_metadata(&p).map_err(|e|e.to_string())?;
        if meta.file_type().is_symlink(){continue;}
        // Live ComfyUI databases, locks and temporary outputs are runtime state, not
        // project knowledge. Index catalogued assets and their sources separately.
        if p.starts_with(root.join("content/runtime")){continue;}
        let name=p.file_name().unwrap().to_string_lossy();
        if ["native","snapshots","target",".git",".venv","__pycache__",".cargo-cache"].contains(&name.as_ref()){continue;}
        if meta.is_dir(){collect(root,&p,out)?;} else if p.starts_with(root){out.push(p);}
    } Ok(())
}
fn source(db:&Db,id:&str,category:&str,text:&str,digest:&str)->Result<(),String>{
    let lines:Vec<&str>=text.lines().collect();
    insert(db,"sources",&[id.into(),category.into(),digest.into(),lines.len().to_string(),text.into()])?;
    for (i,chunk) in lines.chunks(70).enumerate(){
        let body=chunk.join("\n");let start=i*70+1;let end=start+chunk.len()-1;
        entity(db,&format!("source:{id}:{start}"),"source_chunk",&format!("{id}:{start}-{end}"),"source_snapshot","",id,start,&json!({"start_line":start,"end_line":end,"sha256":digest,"category":category}),&body)?;
    } Ok(())
}
fn leaf_params(db:&Db,owner:&str,path:&str,value:&Value)->Result<(),String>{
    match value{
        Value::Object(map)=>for(k,v)in map{leaf_params(db,owner,&if path.is_empty(){k.clone()}else{format!("{path}.{k}")},v)?;},
        Value::Array(a)=>for(i,v)in a.iter().enumerate(){leaf_params(db,owner,&format!("{path}[{i}]"),v)?;},
        _=>insert(db,"parameters",&[owner.into(),path.into(),value.to_string()])?,
    } Ok(())
}
fn map_kind(section:&str,key:&str)->&'static str{match section{"voelker"=>"faction","gebaeude"=>"building","forschung"=>"technology","einheiten"=>if Einheit::aus_name(key).map(|e|e.ist_schiff()).unwrap_or(false){"ship"}else{"defense"},"zonen"=>"planet_zone",_=>"rule"}}
fn target_for(category:&str,key:&str)->Option<String>{
    match category{
        "buildings" if Gebaeude::aus_name(key).is_some()=>Some(format!("building:{key}")),
        "ships" if Einheit::aus_name(key).is_some()=>Some(format!("ship:{key}")),
        "defenses" if Einheit::aus_name(key).is_some()=>Some(format!("defense:{key}")),
        "research" if Forschung::aus_name(key).is_some()=>Some(format!("technology:{key}")),
        "resources" if Gut::aus_name(key).is_some()=>Some(format!("resource:{key}")),
        "portraits"|"emblems" if ["aurelianer","krath","veyari","syntheten"].contains(&key)=>Some(format!("faction:{key}")),
        "missions"=>Some(format!("mission:{key}")),_=>None
    }
}

/// Builds an immutable transactional snapshot, then publishes current.json.
/// Old snapshots remain usable by already connected readers.
pub fn rebuild(root:&Path,pointer:&Path)->Result<Value,String>{
    let started=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_millis();
    let rules_text=read(&root.join("regeln/regelwerk.ron"))?;
    let rules=Regelwerk::laden(&rules_text)?;
    let catalog=json_file(&root.join("content/catalog.json"))?;
    let dir=root.join("wissen/snapshots");fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let path=dir.join(format!("sternenepoche-{started}.duckdb"));
    let db=Db::open(&path,false)?;
    for sql in [
        "CREATE TABLE meta(key VARCHAR PRIMARY KEY,value VARCHAR NOT NULL)",
        "CREATE TABLE sources(id VARCHAR PRIMARY KEY,category VARCHAR,sha256 VARCHAR,line_count BIGINT,body VARCHAR)",
        "CREATE TABLE entities(id VARCHAR PRIMARY KEY,kind VARCHAR,title VARCHAR,status VARCHAR,faction VARCHAR,source VARCHAR,source_line BIGINT,data VARCHAR,body VARCHAR)",
        "CREATE TABLE relations(source_id VARCHAR,relation VARCHAR,target_id VARCHAR,details VARCHAR)",
        "CREATE TABLE parameters(entity_id VARCHAR,path VARCHAR,value VARCHAR)",
        "CREATE TABLE costs(entity_id VARCHAR,resource VARCHAR,base_amount BIGINT,unit VARCHAR)",
        "CREATE TABLE assets(id VARCHAR PRIMARY KEY,category VARCHAR,entity_key VARCHAR,faction VARCHAR,status VARCHAR,path VARCHAR,sha256 VARCHAR,mime_type VARCHAR,data VARCHAR)",
        "CREATE TABLE images(path VARCHAR PRIMARY KEY,sha256 VARCHAR,mime_type VARCHAR,bytes BLOB)",
        "CREATE TABLE issues(id VARCHAR PRIMARY KEY,title VARCHAR,status VARCHAR,owner VARCHAR,source VARCHAR,details VARCHAR)",
        "CREATE TABLE artifacts(path VARCHAR PRIMARY KEY,kind VARCHAR,sha256 VARCHAR,bytes BIGINT)",
        "CREATE INDEX entity_kind ON entities(kind)","CREATE INDEX entity_status ON entities(status)","CREATE INDEX asset_key ON assets(entity_key)","CREATE INDEX relation_source ON relations(source_id)","CREATE INDEX relation_target ON relations(target_id)",
    ]{db.exec(sql,&[])?;}
    db.exec("BEGIN TRANSACTION",&[])?;
    let mut files=Vec::new();
    for dir in ["crates","regeln","content","konfig","wissen/quellen","orchestrator","tools","docs","betrachter"]{let p=root.join(dir);if p.exists(){collect(root,&p,&mut files)?;}}
    for name in ["README.md","COORDINATION.md","Cargo.toml","Cargo.lock","wissen/OPUS_HANDOFF.md","wissen/knowledge.json","wissen/README.md","wissen/verification.json","wissen/native/provenance.json"]{let p=root.join(name);if p.exists(){files.push(p);}}
    // Performance reports and archive inventory are included, prompts/world saves stay
    // separate training artifacts (not mixed into authoritative design knowledge).
    let runs=root.join("laeufe");if runs.exists(){let mut all=Vec::new();collect(root,&runs,&mut all)?;for p in all{let n=p.file_name().unwrap().to_string_lossy();let ext=p.extension().and_then(|v|v.to_str()).unwrap_or("");if ["md","csv"].contains(&ext)||["schluss.json","fortschritt.json","manifest.json"].contains(&n.as_ref()){files.push(p);}else if ["parquet","bin"].contains(&ext){let meta=fs::metadata(&p).map_err(|e|e.to_string())?;insert(&db,"artifacts",&[p.strip_prefix(root).unwrap().to_string_lossy().replace('\\',"/"),ext.into(),String::new(),meta.len().to_string()])?;}}}
    files.sort();files.dedup();let mut fingerprint=Sha256::new();let mut image_count=0;
    for p in files{
        let id=p.strip_prefix(root).map_err(|e|e.to_string())?.to_string_lossy().replace('\\',"/");
        let ext=p.extension().and_then(|s|s.to_str()).unwrap_or("");
        let allowed=["rs","ron","md","json","toml","txt","js","html","css","py","csv","lock"];
        if !allowed.contains(&ext)&&!["png","jpg","jpeg","webp","svg"].contains(&ext){continue;}
        let bytes=fs::read(&p).map_err(|e|format!("{id}: {e}"))?;let digest=hash(&bytes);
        fingerprint.update(id.as_bytes());fingerprint.update(digest.as_bytes());
        if ["png","jpg","jpeg","webp","svg"].contains(&ext){
            let mime=match ext{"png"=>"image/png","webp"=>"image/webp","svg"=>"image/svg+xml",_=>"image/jpeg"};
            db.exec("INSERT INTO images VALUES (?,?,?,from_base64(?))",&[&id,&digest,mime,&STANDARD.encode(&bytes)])?;image_count+=1;
            entity(&db,&format!("image:{id}"),"image",&id,"file_present","",&id,0,&json!({"sha256":digest,"bytes":bytes.len(),"mime_type":mime}),"Vorhandene Bilddatei, Qualitätsabnahme nur wenn gesondert belegt.")?;
        }else{
            let text=String::from_utf8(bytes).map_err(|e|format!("Non-UTF8 source {id}: {e}"))?;
            let category=if id.starts_with("wissen/quellen/"){"original_concept"}else if id.starts_with("crates/"){"rust_source"}else if id.contains("workflows/"){"prepared_workflow"}else if id.starts_with("laeufe/"){"run_evidence"}else if id.starts_with("docs/"){"documentation"}else{"project_source"};
            source(&db,&id,category,&text,&digest)?;
        }
    }
    let mut r=serde_json::to_value(&rules).map_err(|e|e.to_string())?;r.as_object_mut().unwrap().remove("cache");
    for(section,value)in r.as_object().unwrap(){
        if ["voelker","gebaeude","forschung","einheiten","zonen"].contains(&section.as_str()){
            for(key,v)in value.as_object().unwrap(){
                let kind=map_kind(section,key);let id=format!("{kind}:{key}");
                let line=rules_text.lines().position(|line|line.trim_start().starts_with(&format!("{key}:"))).unwrap_or(0)+1;
                let mut data=v.clone();if kind=="faction"{data["visual_design"]=catalog["factions"][key].clone();}
                entity(&db,&id,kind,key,"implemented",if kind=="faction"{key}else{""},"regeln/regelwerk.ron",line,&data,&data.to_string())?;
                leaf_params(&db,&id,"",v)?;
                if let Some(costs)=v["kosten"].as_object(){for(g,n)in costs{insert(&db,"costs",&[id.clone(),g.clone(),n.to_string(),"whole_resource_units_base_level_1".into()])?;relation(&db,&id,"costs",&format!("resource:{g}"),json!({"base_amount":n,"factor":v["faktor"]}))?;}}
                if let Some(needs)=v["braucht"].as_object(){for(g,n)in needs{relation(&db,&id,"requires",&format!("building:{g}"),json!({"level":n}))?;}}
                if let Some(tier)=v["ab_stufe"].as_u64(){relation(&db,&id,"requires",&format!("progression:{tier}"),json!({"stage":tier}))?;}
                for(field,target)in [("labor","building:labor"),("werft","building:werft")]{if let Some(n)=v[field].as_u64(){if n>0{relation(&db,&id,"requires",target,json!({"level":n}))?;}}}
                if let Some(drive)=v["antrieb"].as_str(){relation(&db,&id,"uses_drive",&format!("technology:{drive}"),json!({"note":"Antriebsbezug; Mindeststufe nur wenn zusätzliche Regel vorhanden."}))?;}
                if let Some(fire)=v["schnellfeuer"].as_object(){for(target,n)in fire{let tk=map_kind("einheiten",target);relation(&db,&id,"rapidfire",&format!("{tk}:{target}"),json!({"factor":n}))?;}}
            }
        }else if section=="stufen"{
            for(i,v)in value.as_array().unwrap().iter().enumerate(){let id=format!("progression:{}",i+1);entity(&db,&id,"progression",v["name"].as_str().unwrap_or("Stufe"),"implemented","","regeln/regelwerk.ron",1,v,&v.to_string())?;leaf_params(&db,&id,"",v)?;
                for(field,kind)in [("gebaeude","building"),("forschung","technology")]{if let Some(needs)=v[field].as_object(){for(key,n)in needs{relation(&db,&id,"requires",&format!("{kind}:{key}"),json!({"level":n}))?;}}}
            }
        }else{let id=format!("rules:{section}");entity(&db,&id,"rule_section",section,"implemented","","regeln/regelwerk.ron",1,value,&value.to_string())?;leaf_params(&db,&id,"",value)?;}
    }
    for g in Gut::ALLE{let id=format!("resource:{}",g.name());entity(&db,&id,"resource",g.name(),"implemented","","crates/kern/src/typen.rs",1,&json!({"name":g.name(),"internal_scale":kern::M,"index":g.idx(),"role":"storable_resource"}),"Mengen im Kern als ganzzahlige Tausendstel; Preise im Regelwerk in ganzen Einheiten.")?;}
    for role in [Rolle::Stratege,Rolle::Verwalter,Rolle::Feldherr,Rolle::Diplomat]{let id=format!("role:{}",role.name());let actions=kern::aktion::erlaubte_typen(role);entity(&db,&id,"role",role.name(),"implemented","","crates/kern/src/aktion.rs",1,&json!({"actions":actions,"response_schema":kern::aktion::antwortschema(role)}),&kern::regeltext::regeltext(&rules,role))?;for a in actions{relation(&db,&id,"may_execute",&format!("action:{a}"),json!({}))?;}}
    for action in kern::aktion::erlaubte_typen(Rolle::Alle){entity(&db,&format!("action:{action}"),"action",action,"implemented","","crates/kern/src/aktion.rs",1,&json!({"action":action,"validation":"kern::Welt::handeln", "schema_source":"role entities"}),action)?;}
    for m in kern::Mission::ALLE{entity(&db,&format!("mission:{}",m.name()),"mission",m.name(),"implemented","","crates/kern/src/flotte.rs",1,&json!({"hostile":m.feindlich()}),m.name())?;}
    for a in catalog["assets"].as_array().ok_or("catalog.assets missing")?{
        let s=|key:&str|a[key].as_str().unwrap_or("");let id=s("id");let path=format!("content/{}",s("file"));
        let images=db.rows("SELECT sha256,mime_type FROM images WHERE path=?",&[&path])?;
        let status=if images.is_empty(){"planned_not_generated"}else{"file_present_review_pending"};
        let sha=images.first().and_then(|v|v["sha256"].as_str()).unwrap_or("");
        insert(&db,"assets",&[id.into(),s("category").into(),s("key").into(),s("faction").into(),status.into(),path.clone(),sha.into(),"image/png".into(),a.to_string()])?;
        entity(&db,&format!("asset:{id}"),"asset",s("label"),status,s("faction"),"content/catalog.json",1,a,s("prompt"))?;
        if let Some(target)=target_for(s("category"),s("key")){relation(&db,&format!("asset:{id}"),"depicts",&target,json!({"variant":s("faction")}))?;}
        if !s("faction").is_empty()&&s("faction")!="neutral"{relation(&db,&format!("asset:{id}"),"faction_style",&format!("faction:{}",s("faction")),json!({}))?;}
        for profile in ["pilot","final"]{let workflow=format!("content/workflows/{profile}/{id}.json");relation(&db,&format!("asset:{id}"),"workflow",&format!("source:{workflow}"),json!({"path":workflow,"profile":profile,"exists":root.join(&workflow).exists()}))?;}
    }
    let knowledge=root.join("wissen/knowledge.json");if knowledge.exists(){for record in json_file(&knowledge)?.as_array().ok_or("knowledge.json must be array")?{let s=|key:&str|record[key].as_str().unwrap_or("");entity(&db,s("id"),s("kind"),s("title"),s("status"),"",s("source"),record["source_line"].as_u64().unwrap_or(1)as usize,record,s("body"))?;if ["issue","requirement","decision","handoff"].contains(&s("kind")){insert(&db,"issues",&[s("id").into(),s("title").into(),s("status").into(),s("owner").into(),s("source").into(),record.to_string()])?;}}}
    let snapshot_id=format!("{:x}",fingerprint.finalize());
    for(key,value)in [("schema_version","1".into()),("snapshot_id",snapshot_id.clone()),("created_unix_ms",started.to_string()),("rule_sha256",rules.hash.clone()),("catalog_rule_sha256",catalog["source_rule_sha256"].as_str().unwrap_or("").into()),("root",root.display().to_string()),("database",path.display().to_string()),("gpu_permission","not_granted_by_knowledge_database".into()),("scope","project knowledge; no private account data; training logs inventoried separately".into())]{insert(&db,"meta",&[key.into(),value])?;}
    db.exec("COMMIT",&[])?;db.exec("CHECKPOINT",&[])?;
    let stats=query(&db,"status",&json!({}))?;drop(db);
    // Detect a concurrently edited source instead of publishing a mixed rule/catalog state.
    if read(&root.join("regeln/regelwerk.ron"))?!=rules_text{return Err(format!("Rules changed during index build; unpublished snapshot retained: {}. Rebuild.",path.display()));}
    let publish=json!({"database":path,"snapshot_id":snapshot_id,"created_unix_ms":started,"embedded_images":image_count});
    let temp=pointer.with_extension("json.new");fs::write(&temp,serde_json::to_vec_pretty(&publish).unwrap()).map_err(|e|e.to_string())?;
    // std::fs::rename uses replace-existing semantics on Windows and POSIX.
    fs::rename(&temp,pointer).map_err(|e|format!("Snapshot built but pointer not published: {e}"))?;
    Ok(stats)
}

fn parse_rows(rows:Vec<Value>)->Vec<Value>{rows.into_iter().map(|mut row|{for key in ["data","details"]{if let Some(s)=row[key].as_str(){if let Ok(v)=serde_json::from_str::<Value>(s){row[key]=v;}}}row}).collect()}
fn arg<'a>(a:&'a Value,k:&str)->&'a str{a[k].as_str().unwrap_or("")}
fn paging(a:&Value)->Result<(usize,usize),String>{let number=|k:&str,default:u64|->Result<u64,String>{match a.get(k){None=>Ok(default),Some(v)=>v.as_u64().or_else(||v.as_str().and_then(|s|s.parse().ok())).ok_or_else(||format!("{k} must be unsigned integer"))}};let limit=number("limit",30)?;let offset=number("offset",0)?;if limit==0||limit>200||offset>1_000_000{return Err("limit must be 1..200, offset <=1000000".into());}Ok((limit as usize,offset as usize))}
fn packet(rows:Vec<Value>,limit:usize,offset:usize)->Value{let more=rows.len()>limit;let items:Vec<Value>=parse_rows(rows).into_iter().take(limit).collect();json!({"items":items,"offset":offset,"limit":limit,"next_offset":if more{Some(offset+limit)}else{None}})}
pub fn query(db:&Db,operation:&str,args:&Value)->Result<Value,String>{
    if !args.is_object(){return Err("args must be an object".into());}
    let(lim,off)=paging(args)?;
    match operation{
        "status"=>{let meta=db.rows("SELECT key,value FROM meta ORDER BY key",&[])?;let mut m=serde_json::Map::new();for r in meta{m.insert(r["key"].as_str().unwrap().into(),r["value"].clone());}
            let mut counts=BTreeMap::new();for table in ["sources","entities","relations","parameters","costs","assets","images","issues","artifacts"]{let rows=db.rows(&format!("SELECT count(*) AS n FROM {table}"),&[])?;counts.insert(table,rows[0]["n"].as_str().unwrap().parse::<u64>().unwrap());}
            Ok(json!({"metadata":m,"counts":counts,"entity_kinds":db.rows("SELECT kind,count(*) AS count FROM entities GROUP BY kind ORDER BY kind",&[])?,"asset_status":db.rows("SELECT status,count(*) AS count FROM assets GROUP BY status ORDER BY status",&[])?,"operations":["status","search","entities","entity","source","assets","asset","relations","issues","handoff"],"notes":["Immutable snapshot; status/source hashes identify its exact state.","Images may be planned only. File presence is not proof of visual review.","Interpretations in knowledge records cite evidence; source chunks preserve historical statements."]}))
        },
        "entities"|"search"=>{
            let mut conditions=Vec::new();let mut values:Vec<String>=Vec::new();
            for key in ["kind","faction","status"]{if !arg(args,key).is_empty(){conditions.push(format!("{key}=?"));values.push(arg(args,key).into());}}
            let q=arg(args,"q");if q.chars().count()>300{return Err("q is limited to 300 characters".into());}
            for token in q.split_whitespace().take(10){conditions.push("contains(lower(id || ' ' || title || ' ' || body), lower(?))".into());values.push(token.into());}
            let filter=if conditions.is_empty(){String::new()}else{format!("WHERE {}",conditions.join(" AND "))};
            let refs:Vec<&str>=values.iter().map(String::as_str).collect();
            let sql=format!("SELECT id,kind,title,status,faction,source,source_line,substring(body,1,1600) AS excerpt FROM entities {filter} ORDER BY CASE WHEN kind='source_chunk' THEN 1 ELSE 0 END,kind,id LIMIT {} OFFSET {off}",lim+1);
            Ok(packet(db.rows(&sql,&refs)?,lim,off))
        },
        "entity"=>{let id=arg(args,"id");let mut rows=parse_rows(db.rows("SELECT * FROM entities WHERE id=?",&[id])?);let mut value=rows.pop().ok_or("Entity not found")?;
            value["parameters"]=json!(parse_rows(db.rows("SELECT path,value FROM parameters WHERE entity_id=? ORDER BY path",&[id])?));
            value["costs"]=json!(db.rows("SELECT resource,base_amount,unit FROM costs WHERE entity_id=? ORDER BY resource",&[id])?);
            value["relations"]=json!(parse_rows(db.rows("SELECT * FROM relations WHERE source_id=? OR target_id=? ORDER BY relation,source_id,target_id LIMIT 500",&[id,id])?));Ok(value)
        },
        "source"=>{let id=arg(args,"id").strip_prefix("source:").unwrap_or(arg(args,"id"));let rows=db.rows("SELECT * FROM sources WHERE id=?",&[id])?;let row=rows.first().ok_or("Source not found; id is a source path, not a filesystem path")?;let text=row["body"].as_str().unwrap();let lines:Vec<&str>=text.lines().collect();let end=(off+lim).min(lines.len());let selected=lines.get(off..end).unwrap_or(&[]);Ok(json!({"id":id,"sha256":row["sha256"],"line_count":lines.len(),"start_line":off+1,"end_line":end,"text":selected.join("\n"),"next_offset":if end<lines.len(){Some(end)}else{None}}))},
        "assets"=>{let mut where_parts=Vec::new();let mut values=Vec::new();for(field,col)in[("category","category"),("faction","faction"),("status","status"),("key","entity_key")]{if !arg(args,field).is_empty(){where_parts.push(format!("{col}=?"));values.push(arg(args,field));}}
            let filter=if where_parts.is_empty(){String::new()}else{format!("WHERE {}",where_parts.join(" AND "))};Ok(packet(db.rows(&format!("SELECT id,category,entity_key,faction,status,path,sha256 FROM assets {filter} ORDER BY category,id LIMIT {} OFFSET {off}",lim+1),&values)?,lim,off))},
        "asset"=>{let id=arg(args,"id");let include=args["include_image"].as_bool().unwrap_or(false);let mut value=if let Some(path)=id.strip_prefix("image:"){let rows=db.rows("SELECT path,sha256,mime_type,octet_length(bytes) AS byte_count FROM images WHERE path=?",&[path])?;let mut v=rows.into_iter().next().ok_or("Image not found")?;v["status"]=json!("file_present");v}else{parse_rows(db.rows("SELECT * FROM assets WHERE id=?",&[id.strip_prefix("asset:").unwrap_or(id)])?).into_iter().next().ok_or("Asset not found")?};
            if include{let path=value["path"].as_str().unwrap_or("");let rows=db.rows("SELECT to_base64(bytes) AS base64,mime_type FROM images WHERE path=? AND octet_length(bytes)<=10485760",&[path])?;if let Some(row)=rows.first(){value["base64"]=row["base64"].clone();value["mime_type"]=row["mime_type"].clone();}else{value["image_unavailable"]=json!("Not generated or exceeds 10 MiB inline limit; metadata is retained.");}}Ok(value)},
        "relations"=>{let id=arg(args,"id");let direction=arg(args,"direction");let predicate=match direction{"out"=>"source_id=?","in"=>"target_id=?",""|"both"=>"(source_id=? OR target_id=?)",_=>return Err("direction must be in, out or both".into())};let params=if direction=="out"||direction=="in"{vec![id]}else{vec![id,id]};Ok(packet(db.rows(&format!("SELECT * FROM relations WHERE {predicate} ORDER BY relation,source_id,target_id LIMIT {} OFFSET {off}",lim+1),&params)?,lim,off))},
        "issues"=>{let mut cond=Vec::new();let mut params=Vec::new();for k in ["status","owner"]{if !arg(args,k).is_empty(){cond.push(format!("{k}=?"));params.push(arg(args,k));}}let filter=if cond.is_empty(){String::new()}else{format!("WHERE {}",cond.join(" AND "))};Ok(packet(db.rows(&format!("SELECT * FROM issues {filter} ORDER BY status,owner,id LIMIT {} OFFSET {off}",lim+1),&params)?,lim,off))},
        "handoff"=>{let docs=db.rows("SELECT id,sha256,body FROM sources WHERE id IN ('wissen/OPUS_HANDOFF.md','docs/README.md','COORDINATION.md') ORDER BY id DESC",&[])?;Ok(json!({"status":query(db,"status",&json!({}))?,"documents":docs,"open_items":query(db,"issues",&json!({"limit":200}))?,"reading_order":["Read docs/README.md first: index of the documentation (specification, agent interface, architecture, operations, balance, live test, real run). Historical COORDINATION statements are timestamped and can be superseded.","Use entities kind=faction/building/technology/ship/defense then entity(id) for full details and edges.","Use source(id,offset,limit) for exact evidence; paginate until next_offset=null.","Use assets then asset(id,include_image=true); do not confuse pending prompts with generated images.","Record explicit acknowledgment including snapshot_id in COORDINATION.md."]}))},
        _=>Err(format!("Unknown operation {operation}")),
    }
}
