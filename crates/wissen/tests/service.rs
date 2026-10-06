use serde_json::{json,Value};
use wissen::service::{http_request,tools_schema,validate_args,McpSession,MAX_REQUEST,MAX_RESPONSE};

fn headers()->Vec<(String,String)>{vec![("Host".into(),"127.0.0.1:8197".into()),("Content-Type".into(),"application/json".into())]}
fn request(id:i64,method:&str,params:Value)->Value{json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})}
fn initialize(s:&mut McpSession)->Value {
    let reply=s.dispatch(request(1,"initialize",json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}})),|_,_|panic!("initialization must not query")).unwrap();
    assert!(s.dispatch(json!({"jsonrpc":"2.0","method":"notifications/initialized"}),|_,_|panic!()).is_none());
    reply
}

#[test]
fn http_routes_only_valid_bounded_read_operations() {
    let reply=http_request("GET","/v1/search?q=Erzmine%20Stufe&limit=20&offset=0",&headers(),b"",8197,|op,args|{
        assert_eq!(op,"search");assert_eq!(args["q"],"Erzmine Stufe");assert_eq!(args["limit"],20);Ok(json!({"items":[]}))
    });assert_eq!(reply.status,200);
    let body=br#"{"operation":"entity","args":{"id":"building:erzmine"}}"#;
    let reply=http_request("POST","/v1/query",&headers(),body,8197,|op,args|{assert_eq!(op,"entity");Ok(args.clone())});
    assert_eq!(reply.status,200);
    for url in ["/v1/sql?q=DROP+TABLE+assets","/v1/source?id=x&path=C%3A%5Csecret","/v1/search?q=x&limit=99999","/v1/search?q=x&q=y","/v1/search?q=%ZZ","/v1/search?q=%FF","/v1/search"] {
        assert_eq!(http_request("GET",url,&headers(),b"",8197,|_,_|panic!("invalid request must not query")).status,400,"{url}");
    }
    assert_eq!(http_request("DELETE","/v1/entity?id=x",&headers(),b"",8197,|_,_|panic!()).status,405);
}

#[test]
fn http_rejects_cross_origin_dns_rebinding_and_oversized_inputs() {
    for bad in ["http://evil.example","null","http://127.0.0.1:8196"] {
        let mut h=headers();h.push(("Origin".into(),bad.into()));
        assert_eq!(http_request("GET","/v1/status",&h,b"",8197,|_,_|panic!()).status,403);
    }
    for host in ["evil.example:8197","127.0.0.1:8888",""] {
        let mut h=headers();h[0].1=host.into();
        assert_eq!(http_request("GET","/v1/status",&h,b"",8197,|_,_|panic!()).status,403);
    }
    let mut h=headers();h.push(("Origin".into(),"http://127.0.0.1:8197".into()));
    assert_eq!(http_request("GET","/v1/status",&h,b"",8197,|_,_|Ok(json!({}))).status,200);
    assert_eq!(http_request("POST","/v1/query",&headers(),&vec![b'x';MAX_REQUEST+1],8197,|_,_|panic!()).status,413);
    assert_eq!(http_request("GET","/v1/status",&headers(),b"",8197,|_,_|Ok(json!("x".repeat(MAX_RESPONSE+1)))).status,413);
}

#[test]
fn schemas_disallow_sql_paths_and_out_of_range_pagination() {
    let tools=tools_schema();assert_eq!(tools["tools"].as_array().unwrap().len(),10);
    for t in tools["tools"].as_array().unwrap(){assert_eq!(t["annotations"]["readOnlyHint"],true);assert_eq!(t["inputSchema"]["additionalProperties"],false);}
    assert!(validate_args("status",&json!({"sql":"select 1"})).is_err());
    assert!(validate_args("source",&json!({"id":"known-id","limit":201})).is_err());
    assert!(validate_args("asset",&json!({"id":"known-id","include_image":"true"})).is_err());
    assert!(validate_args("relations",&json!({"id":"known-id","direction":"both"})).is_ok());
}

#[test]
fn mcp_initialization_notifications_and_protocol_errors() {
    let mut s=McpSession::default();
    assert_eq!(s.dispatch(request(1,"tools/list",json!({})),|_,_|panic!()).unwrap()["error"]["code"],-32002);
    let reply=initialize(&mut s);assert_eq!(reply["result"]["protocolVersion"],"2025-06-18");
    assert!(reply["result"]["capabilities"]["tools"].is_object());
    assert_eq!(s.dispatch(request(2,"initialize",json!({})),|_,_|panic!()).unwrap()["error"]["code"],-32600);
    assert_eq!(s.dispatch(json!([request(3,"ping",json!({}))]),|_,_|panic!()).unwrap()["error"]["code"],-32600);
    assert_eq!(s.dispatch(request(4,"unknown",json!({})),|_,_|panic!()).unwrap()["error"]["code"],-32601);
    assert!(s.dispatch(json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":5}}),|_,_|panic!()).is_none());
    assert!(s.dispatch(request(6,"ping",json!({})),|_,_|panic!()).unwrap()["result"].is_object());
}

#[test]
fn mcp_tools_use_read_only_query_and_report_tool_failures() {
    let mut s=McpSession::default();initialize(&mut s);
    let reply=s.dispatch(request(3,"tools/call",json!({"name":"entity","arguments":{"id":"erzmine"}})),|op,args|{assert_eq!(op,"entity");Ok(args.clone())}).unwrap();
    assert_eq!(reply["result"]["isError"],false);
    assert!(reply["result"]["content"][0]["text"].as_str().unwrap().contains("erzmine"));
    let reply=s.dispatch(request(4,"tools/call",json!({"name":"entity","arguments":{"id":"absent"}})),|_,_|Err("Objekt nicht vorhanden".into())).unwrap();
    assert_eq!(reply["result"]["isError"],true);
    assert_eq!(s.dispatch(request(5,"tools/call",json!({"name":"sql","arguments":{}})),|_,_|panic!()).unwrap()["error"]["code"],-32602);
}

#[test]
fn mcp_resources_templates_images_and_handoff_are_real_query_results() {
    let mut s=McpSession::default();initialize(&mut s);
    let reply=s.dispatch(request(2,"resources/templates/list",json!({})),|_,_|panic!()).unwrap();
    assert_eq!(reply["result"]["resourceTemplates"].as_array().unwrap().len(),4);
    let reply=s.dispatch(request(3,"resources/read",json!({"uri":"sternenepoche://source/crates%2Fkern%2Fsrc%2Fwelt.rs"})),|op,args|{assert_eq!(op,"source");assert_eq!(args["id"],"crates/kern/src/welt.rs");Ok(json!({"text":"indexed"}))}).unwrap();
    assert_eq!(reply["result"]["contents"][0]["mimeType"],"application/json");
    let reply=s.dispatch(request(4,"resources/read",json!({"uri":"sternenepoche://asset-image/ship"})),|op,args|{assert_eq!(op,"asset");assert_eq!(args["include_image"],true);Ok(json!({"base64":"aW1hZ2U=","mime_type":"image/png"}))}).unwrap();
    assert_eq!(reply["result"]["contents"][0]["blob"],"aW1hZ2U=");
    let reply=s.dispatch(request(5,"resources/read",json!({"uri":"sternenepoche://asset-image/planned"})),|_,_|Ok(json!({"status":"planned"}))).unwrap();
    assert_eq!(reply["error"]["code"],-32002);
    assert_eq!(s.dispatch(request(6,"resources/read",json!({"uri":"file:///C:/secrets"})),|_,_|panic!()).unwrap()["error"]["code"],-32602);
    let reply=s.dispatch(request(7,"prompts/get",json!({"name":"opus_handoff"})),|op,_|{assert_eq!(op,"handoff");Ok(json!({"owner":"Claude","task":"balance"}))}).unwrap();
    assert!(reply["result"]["messages"][0]["content"]["text"].as_str().unwrap().contains("balance"));
}
