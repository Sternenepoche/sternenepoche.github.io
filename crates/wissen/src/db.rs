//! Small owned wrapper over the pinned DuckDB 1.5.6 stable C ABI.
//! No Python subprocess, SQL shell or extension download is used.
use libloading::Library;
use std::{ffi::{c_char,c_void,CStr,CString}, path::Path, ptr};
use serde_json::{Value,Map};

type Handle=*mut c_void;
#[repr(C)]
#[derive(Default)]
struct ResultSet { columns:u64, rows:u64, changed:u64, column_data:Handle, error:*mut c_char, internal:Handle }
type State=u32;
struct Api {
    _library:Library,
    open:unsafe extern "C" fn(*const c_char,*mut Handle,Handle,*mut *mut c_char)->State,
    close:unsafe extern "C" fn(*mut Handle),
    connect:unsafe extern "C" fn(Handle,*mut Handle)->State,
    disconnect:unsafe extern "C" fn(*mut Handle),
    config_create:unsafe extern "C" fn(*mut Handle)->State,
    config_set:unsafe extern "C" fn(Handle,*const c_char,*const c_char)->State,
    config_destroy:unsafe extern "C" fn(*mut Handle),
    prepare:unsafe extern "C" fn(Handle,*const c_char,*mut Handle)->State,
    prepare_error:unsafe extern "C" fn(Handle)->*const c_char,
    prepare_destroy:unsafe extern "C" fn(*mut Handle),
    bind:unsafe extern "C" fn(Handle,u64,*const c_char,u64)->State,
    execute:unsafe extern "C" fn(Handle,*mut ResultSet)->State,
    destroy:unsafe extern "C" fn(*mut ResultSet),
    error:unsafe extern "C" fn(*mut ResultSet)->*const c_char,
    rows:unsafe extern "C" fn(*mut ResultSet)->u64,
    columns:unsafe extern "C" fn(*mut ResultSet)->u64,
    name:unsafe extern "C" fn(*mut ResultSet,u64)->*const c_char,
    value:unsafe extern "C" fn(*mut ResultSet,u64,u64)->*mut c_char,
    is_null:unsafe extern "C" fn(*mut ResultSet,u64,u64)->bool,
    free:unsafe extern "C" fn(Handle),
}
unsafe fn string(p:*const c_char)->String { if p.is_null(){String::new()} else {CStr::from_ptr(p).to_string_lossy().into_owned()} }
fn c(s:&str)->Result<CString,String>{CString::new(s).map_err(|_|"NUL in SQL/path".into())}
impl Api {
    fn load()->Result<Self,String> {
        let path=std::env::var_os("STERNENEPOCHE_DUCKDB_LIB").map(std::path::PathBuf::from).unwrap_or_else(||Path::new(env!("CARGO_MANIFEST_DIR")).join("../../wissen/native/duckdb-1.5.6/duckdb.dll"));
        // Library lifetime exceeds all copied function pointers and database handles.
        unsafe {
            let lib=Library::new(&path).map_err(|e|format!("DuckDB library {}: {e}",path.display()))?;
            macro_rules! sym {($name:literal)=>{*lib.get(concat!($name,"\0").as_bytes()).map_err(|e|e.to_string())?};}
            Ok(Self{open:sym!("duckdb_open_ext"),close:sym!("duckdb_close"),connect:sym!("duckdb_connect"),disconnect:sym!("duckdb_disconnect"),config_create:sym!("duckdb_create_config"),config_set:sym!("duckdb_set_config"),config_destroy:sym!("duckdb_destroy_config"),prepare:sym!("duckdb_prepare"),prepare_error:sym!("duckdb_prepare_error"),prepare_destroy:sym!("duckdb_destroy_prepare"),bind:sym!("duckdb_bind_varchar_length"),execute:sym!("duckdb_execute_prepared"),destroy:sym!("duckdb_destroy_result"),error:sym!("duckdb_result_error"),rows:sym!("duckdb_row_count"),columns:sym!("duckdb_column_count"),name:sym!("duckdb_column_name"),value:sym!("duckdb_value_varchar"),is_null:sym!("duckdb_value_is_null"),free:sym!("duckdb_free"),_library:lib})
        }
    }
}
pub struct Db { api:Api, db:Handle, conn:Handle }
impl Drop for Db {fn drop(&mut self){unsafe{(self.api.disconnect)(&mut self.conn);(self.api.close)(&mut self.db);}}}
impl Db {
    pub fn open(path:&Path,read_only:bool)->Result<Self,String>{
        let resolved;
        let path=if path.extension().and_then(|s|s.to_str())==Some("json") {
            let v:Value=serde_json::from_slice(&std::fs::read(path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
            resolved=std::path::PathBuf::from(v["database"].as_str().ok_or("current.json missing database path")?);
            resolved.as_path()
        }else{path};
        let api=Api::load()?;
        let path=c(&path.to_string_lossy())?;
        let mut db=ptr::null_mut(); let mut conn=ptr::null_mut(); let mut config=ptr::null_mut();
        unsafe {
            if (api.config_create)(&mut config)!=0{return Err("DuckDB config allocation failed".into());}
            for (key,value) in [("access_mode",if read_only{"READ_ONLY"}else{"READ_WRITE"}),("threads","2"),("memory_limit","512MB"),("enable_external_access","false"),("autoload_known_extensions","false"),("autoinstall_known_extensions","false")] {
                if (api.config_set)(config,c(key)?.as_ptr(),c(value)?.as_ptr())!=0{(api.config_destroy)(&mut config);return Err(format!("DuckDB option {key} failed"));}
            }
            let mut error=ptr::null_mut();
            let state=(api.open)(path.as_ptr(),&mut db,config,&mut error);
            (api.config_destroy)(&mut config);
            if state!=0{let msg=string(error);(api.free)(error.cast());return Err(msg);}
            if (api.connect)(db,&mut conn)!=0{(api.close)(&mut db);return Err("DuckDB connection failed".into());}
        }
        Ok(Self{api,db,conn})
    }
    /// All external values are bound parameters. Results deliberately remain strings;
    /// application queries parse JSON payloads and counts explicitly.
    pub fn rows(&self,sql:&str,params:&[&str])->Result<Vec<Value>,String>{
        let a=&self.api; let sql=c(sql)?; let mut stmt=ptr::null_mut();
        unsafe {
            if (a.prepare)(self.conn,sql.as_ptr(),&mut stmt)!=0{let error=string((a.prepare_error)(stmt));(a.prepare_destroy)(&mut stmt);return Err(error);}
            for (i,v) in params.iter().enumerate(){
                if (a.bind)(stmt,(i+1)as u64,v.as_ptr().cast(),v.len()as u64)!=0{(a.prepare_destroy)(&mut stmt);return Err(format!("DuckDB parameter {} rejected",i+1));}
            }
            let mut result=ResultSet::default();
            let state=(a.execute)(stmt,&mut result); (a.prepare_destroy)(&mut stmt);
            if state!=0{let error=string((a.error)(&mut result));(a.destroy)(&mut result);return Err(error);}
            let names:Vec<String>=(0..(a.columns)(&mut result)).map(|i|string((a.name)(&mut result,i))).collect();
            let count=(a.rows)(&mut result);
            if count>100_000{(a.destroy)(&mut result);return Err("Result exceeds internal row bound".into());}
            let mut rows=Vec::with_capacity(count as usize);
            for row in 0..count{
                let mut values=Map::new();
                for (col,name) in names.iter().enumerate(){
                    let v=if (a.is_null)(&mut result,col as u64,row){Value::Null}else{let p=(a.value)(&mut result,col as u64,row);let text=string(p);(a.free)(p.cast());Value::String(text)};
                    values.insert(name.clone(),v);
                }
                rows.push(Value::Object(values));
            }
            (a.destroy)(&mut result); Ok(rows)
        }
    }
    pub fn exec(&self,sql:&str,params:&[&str])->Result<(),String>{self.rows(sql,params).map(|_|())}
}
