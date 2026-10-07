//! Local browser adapter. Every action goes through the same Session as the native UI.
use std::{io::{Read,Write},net::{TcpListener,TcpStream},path::{Path,PathBuf},time::{Duration,Instant,SystemTime,UNIX_EPOCH}};
use serde_json::{json,Value};
use spieler::Session;

fn response(stream:&mut TcpStream,status:&str,mime:&str,data:&[u8]) -> std::io::Result<()> {
    write!(stream,"HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; frame-ancestors 'none'\r\nConnection: close\r\n\r\n",data.len())?;
    stream.write_all(data)
}
fn request(s:&mut TcpStream)->Result<(String,String,Vec<(String,String)>,Vec<u8>),String>{
    s.set_read_timeout(Some(Duration::from_secs(3))).map_err(|e|e.to_string())?;
    s.set_write_timeout(Some(Duration::from_secs(5))).map_err(|e|e.to_string())?;
    let mut data=Vec::new();let mut buf=[0;4096];
    let end=loop{let n=s.read(&mut buf).map_err(|e|e.to_string())?;if n==0{return Err("Leere Anfrage".into())}data.extend_from_slice(&buf[..n]);if let Some(i)=data.windows(4).position(|b|b==b"\r\n\r\n"){break i+4}if data.len()>16384{return Err("Header zu groß".into())}};
    let header=std::str::from_utf8(&data[..end]).map_err(|e|e.to_string())?;
    let mut lines=header.lines();let first:Vec<_>=lines.next().unwrap_or("").split_whitespace().collect();if first.len()!=3{return Err("Ungültige Anfrage".into())}
    let method=first[0].to_string();let path=first[1].to_string();
    let headers:Vec<_>=lines.filter_map(|s|s.split_once(':').map(|(k,v)|(k.to_lowercase(),v.trim().to_string()))).collect();
    if headers.iter().any(|(k,_)|k=="transfer-encoding") || headers.iter().filter(|(k,_)|k=="content-length").count()>1{return Err("Mehrdeutiger Request".into())}
    let len=headers.iter().find(|(k,_)|k=="content-length").map(|(_,v)|v.parse::<usize>()).transpose().map_err(|e|e.to_string())?.unwrap_or(0);
    if len>65536{return Err("Anfrage zu groß".into())}
    while data.len()<end+len{let n=s.read(&mut buf).map_err(|e|e.to_string())?;if n==0{return Err("Unvollständige Anfrage".into())}data.extend_from_slice(&buf[..n]);}
    Ok((method,path,headers,data[end..end+len].to_vec()))
}
fn origin_ok(headers:&[(String,String)],port:u16,mutation:bool)->bool{
    let header=|key:&str|headers.iter().find(|(k,_)|k==key).map(|(_,v)|v.as_str());
    let host=format!("127.0.0.1:{port}");let origin=format!("http://{host}");
    header("host")==Some(host.as_str()) && header("origin").map(|o|o==origin).unwrap_or(!mutation)
        && (!mutation || (header("x-sternenepoche")==Some("1") && header("content-type")==Some("application/json")))
}
fn state(s:&Session,running:bool,speed:u64)->Value{
    let mut sectors=Vec::new();let r=s.regeln();
    for sector in 1..=r.welt.sektoren {let mut systems=Vec::new();for start in (1..=r.welt.systeme_je_sektor as usize).step_by(20) {
        if let Ok(q)=s.query(&json!({"typ":"galaxie","sektor":sector,"von":start,"bis":start+19})){systems.extend(q["systeme"].as_array().unwrap().clone());}
    }sectors.push(json!({"sektor":sector,"systeme":systems}));}
    json!({"view":s.view(),"seconds":s.seconds(),"running":running,"speed":speed,"ended":s.ended(),"bots":s.bot_count(),"ki":s.ki_info(),"sectors":sectors,"tree":spieler::technologie::baum(s)})
}
fn main()->Result<(),Box<dyn std::error::Error>>{
    let args:Vec<_>=std::env::args().collect();let arg=|key:&str|args.windows(2).find(|a|a[0]==key).map(|a|a[1].clone());
    let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
    let port=arg("--port").unwrap_or("8787".into()).parse::<u16>()?;
    let listener=TcpListener::bind(("127.0.0.1",port))?;listener.set_nonblocking(true)?;
    let autosave=root.join("saves/browser-autosave.bin");
    let mut s=if let Some(path)=arg("--load"){Session::load(Path::new(&path))?}
        else if autosave.exists() && !args.iter().any(|a|a=="--new"){Session::load(&autosave)?}
        else if let Some(config)=arg("--models"){
            let cfg:sternenepoche_agenten::config::Config=serde_json::from_slice(&std::fs::read(&config)?)?;
            cfg.validate()?;
            let folder=root.join(format!("laeufe/browser-modelle-{}",SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()));
            Session::with_models(42,50,0,&spieler::ki_reiche(50,0,4),cfg,&folder,0.)?
        }else{Session::new(42,50,0)?};
    let mut running=false;let mut speed=60u64;let mut tick=Instant::now();let mut accumulator=0.;
    println!("Sternenepoche: http://127.0.0.1:{port} ({} Skriptgegner)",s.bot_count());
    loop {
        s.ki_abholen();
        let dt=tick.elapsed().as_secs_f64();tick=Instant::now();
        if running && !s.ki_denkt(){accumulator+=dt*speed as f64;if accumulator>=s.window_seconds() as f64 {match s.step(){Ok(true)=>{accumulator-=s.window_seconds() as f64;if !s.ki_denkt(){if let Err(e)=s.save_replace(&autosave){eprintln!("Autosave: {e}");running=false;}}},Ok(false)=>{},Err(e)=>{running=false;eprintln!("Spieluhr: {e}");}}}}else{accumulator=0.;}
        let (mut stream,_)=match listener.accept(){Ok(x)=>x,Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>{std::thread::sleep(Duration::from_millis(20));continue},Err(e)=>return Err(e.into())};
        let result=(||->Result<(String,Vec<u8>),String>{
            let(method,path,headers,body)=request(&mut stream)?;
            if !origin_ok(&headers,port,method=="POST"){return Err("Nur die lokale Spieloberfläche darf zugreifen".into())}
            if method=="GET"{
                if path=="/api/state"{return Ok(("application/json".into(),serde_json::to_vec(&state(&s,running,speed)).unwrap()))}
                if path=="/api/catalog"{
                    let c:Value=serde_json::from_slice(&std::fs::read(root.join("content/catalog.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
                    let a:Vec<_>=c["assets"].as_array().unwrap().iter().map(|a|json!({"id":a["id"],"category":a["category"],"key":a["key"],"faction":a["faction"],"label":a["label"],"file":a["file"]})).collect();return Ok(("application/json".into(),serde_json::to_vec(&a).unwrap()));
                }
                let(mime,file)=match path.as_str(){"/"=>("text/html; charset=utf-8",root.join("browser/index.html")),"/app.js"=>("text/javascript; charset=utf-8",root.join("browser/app.js")),"/style.css"=>("text/css; charset=utf-8",root.join("browser/style.css")),p if p.starts_with("/assets/") && p.ends_with(".png") && !p.contains("..") && p.chars().all(|c|c.is_ascii_alphanumeric()||"/_.-".contains(c))=>("image/png",root.join("content").join(&p[1..])),_=>return Err("Unbekannter Pfad".into())};
                let data=std::fs::read(&file).map_err(|e|e.to_string())?;
                if mime=="image/png"{let proof:Value=serde_json::from_slice(&std::fs::read(file.with_extension("provenance.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;if proof["status"]!="complete" || proof["sha256"]!=sternenepoche_agenten::journal::hash(&data){return Err("Bildnachweis stimmt nicht".into())}}
                return Ok((mime.into(),data));
            }
            if method!="POST"{return Err("Methode nicht erlaubt".into())}
            let value:Value=serde_json::from_slice(&body).map_err(|e|e.to_string())?;
            let out=match path.as_str(){
                "/api/query"=>s.query(&value)?,
                "/api/action"=>{let(ok,message)=s.act(value);if ok && !s.ki_denkt(){s.save_replace(&autosave)?;}json!({"ok":ok,"message":message})},
                "/api/clock"=>{if let Some(on)=value["running"].as_bool(){running=on}if let Some(n)=value["speed"].as_u64(){if ![1,60,900,3600].contains(&n){return Err("Ungültiges Tempo".into())}speed=n;}
                    if value["step"]==true {s.step()?;if !s.ki_denkt(){s.save_replace(&autosave)?;}}json!({"ok":true})},
                "/api/save"=>{s.save_replace(&autosave)?;json!({"ok":true,"message":"Partie gespeichert · saves/browser-autosave.bin"})},
                _=>return Err("Unbekannte Aktion".into())};
            Ok(("application/json".into(),serde_json::to_vec(&out).unwrap()))
        })();
        match result{Ok((mime,data))=>{let _=response(&mut stream,"200 OK",&mime,&data);},Err(message)=>{let _=response(&mut stream,"400 Bad Request","application/json",&serde_json::to_vec(&json!({"ok":false,"message":message})).unwrap());}}
    }
}

#[cfg(test)] mod tests {use super::*;#[test]fn rejects_cross_origin_mutations(){let h=vec![("host".into(),"127.0.0.1:8787".into()),("origin".into(),"http://evil.test".into()),("x-sternenepoche".into(),"1".into()),("content-type".into(),"application/json".into())];assert!(!origin_ok(&h,8787,true));let mut good=h;good[1].1="http://127.0.0.1:8787".into();assert!(origin_ok(&good,8787,true));good.remove(2);assert!(!origin_ok(&good,8787,true));}}
