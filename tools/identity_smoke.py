"""Real isolated HTTP/SQLite/TOTP/SMTP checks. No real emails or live-world changes."""
from pathlib import Path
import argparse,base64,hashlib,hmac,json,os,re,socketserver,sqlite3,struct,subprocess,sys,threading,time,urllib.request,urllib.error,uuid
from urllib.parse import urlparse,parse_qs
from email.parser import BytesParser
from email.policy import default

ROOT=Path(__file__).resolve().parents[1]
DATA=ROOT/'laeufe/identity-2026-10-08'/('check-'+uuid.uuid4().hex[:8])
PUBLIC='http://127.0.0.1:18986';ADMIN='http://127.0.0.1:18987'
PASSWORD='isoliertes-authenticator-testpasswort'
messages=[];process=None;key=None

class SMTP(socketserver.StreamRequestHandler):
    def handle(self):
        self.wfile.write(b'220 localhost test SMTP\r\n')
        while line:=self.rfile.readline():
            command=line.split(b' ',1)[0].strip().upper()
            if command in [b'EHLO',b'HELO']:answer=b'250 localhost\r\n'
            elif command in [b'MAIL',b'RCPT',b'RSET',b'NOOP']:answer=b'250 OK\r\n'
            elif command==b'DATA':
                self.wfile.write(b'354 End with a dot\r\n');data=b''
                while (part:=self.rfile.readline()) not in [b'.\r\n',b'']:
                    data+=part[1:] if part.startswith(b'..') else part
                messages.append(BytesParser(policy=default).parsebytes(data));answer=b'250 queued locally\r\n'
            elif command==b'QUIT':self.wfile.write(b'221 bye\r\n');return
            else:answer=b'502 unsupported\r\n'
            self.wfile.write(answer)

def request(path,body=None,token=None,admin=False):
    headers={}
    if body is not None:headers['Content-Type']='application/json'
    if token:headers['X-Sternenepoche-Admin' if admin else 'Authorization']=token if admin else 'Bearer '+token
    req=urllib.request.Request((ADMIN if admin else PUBLIC)+path,data=None if body is None else json.dumps(body).encode(),headers=headers)
    try:
        with urllib.request.urlopen(req,timeout=20) as r:return r.status,json.loads(r.read())
    except urllib.error.HTTPError as e:
        body=e.read()
        return e.code,json.loads(body) if body else {}

def ok(path,body=None,token=None,admin=False,status=200):
    code,value=request(path,body,token,admin);assert code==status,(path,code,value);return value

def start(binary):
    global process,key
    DATA.mkdir(parents=True,exist_ok=True)
    process=subprocess.Popen([str(binary),'--data',str(DATA),'--bind','127.0.0.1:18986','--admin-bind','127.0.0.1:18987'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,creationflags=subprocess.CREATE_NO_WINDOW if sys.platform=='win32' else 0)
    for _ in range(100):
        if process.poll() is not None:raise AssertionError('Isolated identity server failed to start')
        try:
            if request('/api/lobby')[0]==200:break
        except OSError:pass
        time.sleep(.1)
    else:raise AssertionError('Server did not start')
    key=ok('/api/admin/bootstrap',admin=True)['key']

def stop():
    global process
    if process is not None:process.terminate();process.wait(timeout=15);process=None

def admin(action,**fields):
    return ok('/api/admin/action',{'world_id':ok('/api/lobby')['world_id'],'action':action,**fields},key,True)

def otp(uri,offset=0):
    values=parse_qs(urlparse(uri).query);secret=values['secret'][0]
    raw=base64.b32decode(secret+'='*((-len(secret))%8));counter=int(time.time())//30+offset
    data=hmac.new(raw,struct.pack('>Q',counter),hashlib.sha1).digest();index=data[-1]&15
    return f'{(int.from_bytes(data[index:index+4],"big")&0x7fffffff)%1000000:06d}'

def last_email_code():return re.search(r'^([0-9]{6})\r?$',messages[-1].get_content(),re.M)[1]

def browser_checks():
    import cv2
    from playwright.sync_api import sync_playwright,expect
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome' if sys.platform=='win32' else None,headless=True)
        page=browser.new_page(viewport={'width':1440,'height':1000});errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
        page.goto(PUBLIC,wait_until='networkidle');page.locator('#login [name=name]').fill('AuthTester');page.locator('#login [name=password]').fill(PASSWORD);page.locator('#login button').click()
        expect(page.locator('#game')).to_be_visible();page.locator('[data-tab=profil]').click()
        page.locator('#authenticator-panel summary').first.click();page.locator('#authenticator-settings [name=password]').fill(PASSWORD);page.locator('#authenticator-create').click()
        expect(page.locator('#authenticator-qr')).to_be_visible();uri=page.locator('#authenticator-link').get_attribute('href');assert uri.startswith('otpauth://totp/')
        expect(page.locator('#authenticator-qr')).to_have_js_property('complete',True)
        image=DATA/'browser-qr.png';page.locator('#authenticator-qr').screenshot(path=str(image))
        (DATA/'browser-qr-native.png').write_bytes(base64.b64decode(page.locator('#authenticator-qr').get_attribute('src').split(',',1)[1]))
        decoded,_,_=cv2.QRCodeDetector().detectAndDecode(cv2.imread(str(image)));assert decoded==uri,'Displayed QR does not decode to its local otpauth URI'
        assert not page.locator('#authenticator-setup .ai-image-label').count(),'Deterministic security QR must not receive an AI label'
        page.locator('#authenticator-confirm [name=code]').fill(otp(uri));page.locator('#authenticator-confirm button').click();expect(page.locator('#authenticator-recovery')).to_be_visible()
        assert len(page.locator('#authenticator-codes').inner_text().splitlines())==8
        with page.expect_download() as download:page.locator('#authenticator-download').click()
        download.value.save_as(str(DATA/'browser-recovery.txt'))
        page.screenshot(path=str(DATA/'browser-authenticator.png'))
        page.locator('#logout').click();expect(page.locator('#login')).to_be_visible();assert not page.locator('#authenticator-qr').get_attribute('src')
        page.locator('#login [name=name]').fill('AuthTester');page.locator('#login [name=password]').fill(PASSWORD);page.locator('#login [name=otp]').fill(otp(uri,1));page.locator('#login button').click();expect(page.locator('#game')).to_be_visible()
        assert not errors,errors
        page=browser.new_page(viewport={'width':390,'height':844});page.goto(PUBLIC,wait_until='networkidle');page.locator('#registration-flow summary').click();expect(page.locator('#email-send')).to_be_enabled()
        page.locator('#email-start [name=email]').fill('browser@example.org');page.locator('#email-send').click();expect(page.locator('#email-verify')).to_be_visible()
        page.locator('#email-verify [name=code]').fill(last_email_code());page.locator('#email-verify button').click();expect(page.locator('#registration-profile')).to_be_visible()
        form=page.locator('#registration-profile');form.locator('[name=name]').fill('BrowserTester');form.locator('[name=password]').fill(PASSWORD);form.locator('[name=confirm_password]').fill(PASSWORD);form.locator('[name=volk]').select_option('syntheten');form.locator('button').click();expect(page.locator('#game')).to_be_visible()
        assert page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),'Registration causes mobile overflow'
        page=browser.new_page();page.goto(ADMIN,wait_until='networkidle');page.locator('#account-filter').fill('AuthTester');expect(page.locator('#accounts tbody tr')).to_have_count(1)
        page.locator('[data-player-details]').click();expect(page.locator('#account-detail')).to_be_visible();assert 'login' in page.locator('#account-events').inner_text()
        page.screenshot(path=str(DATA/'browser-private-registry.png'));browser.close()

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--browser',action='store_true');parser.add_argument('--server-exe',type=Path,default=Path(os.environ.get('STERNENEPOCHE_SERVER_EXE',str(ROOT/'target/release'/('sternenepoche-server.exe' if sys.platform=='win32' else 'sternenepoche-server')))));args=parser.parse_args()
    smtp=socketserver.ThreadingTCPServer(('127.0.0.1',0),SMTP);smtp.daemon_threads=True;threading.Thread(target=smtp.serve_forever,daemon=True).start()
    try:
        start(args.server_exe);world=ok('/api/lobby')['world_id'];assert ok('/api/lobby')['freigegebene_plaetze']==5
        assert ok('/api/registration/status')['enabled'] is False
        ok('/api/registration/start',{'email':'none@example.org'},status=503);ok('/api/register',{'name':'Bypass','password':PASSWORD},status=503)
        for name in ['AuthTester','TestProbe2','TestProbe3','TestProbe4','TestProbe5']:admin('test_account',name=name,password=PASSWORD)
        ok('/api/admin/action',{'world_id':world,'action':'test_account','name':'TooMany','password':PASSWORD},key,True,status=409)
        session=ok('/api/login',{'name':'AuthTester','password':PASSWORD});token=session['token'];ok('/api/claim',{'world_id':world,'mode':'mensch','volk':'aurelianer'},token)
        setup=ok('/api/authenticator/setup',{'password':PASSWORD},token);uri=setup['otpauth_uri'];code=otp(uri)
        enabled=ok('/api/authenticator/enable',{'password':PASSWORD,'code':code},token);codes=enabled['recovery_codes'];assert len(codes)==8
        ok('/api/login',{'name':'AuthTester','password':PASSWORD},status=401)
        ok('/api/login',{'name':'AuthTester','password':PASSWORD,'otp':code},status=401)
        future=otp(uri,1);second=ok('/api/login',{'name':'AuthTester','password':PASSWORD,'otp':future})
        ok('/api/login',{'name':'AuthTester','password':PASSWORD,'otp':future},status=401)
        ok('/api/login',{'name':'AuthTester','password':PASSWORD,'otp':codes[0]});ok('/api/login',{'name':'AuthTester','password':PASSWORD,'otp':codes[0]},status=401)
        with sqlite3.connect(DATA/'spiel.sqlite3') as db:
            encrypted=db.execute('select totp_secret from accounts where name=?',['AuthTester']).fetchone()[0];assert setup['manual_key'] not in encrypted
            assert db.execute('select count(*) from registered_players').fetchone()[0]==5
        backup=admin('backup')['backup'];assert (DATA/'backups'/Path(backup).with_suffix('.auth-key.txt')).exists()
        stop();start(args.server_exe);assert ok('/api/lobby')['world_id']==world;assert ok('/api/me',token=token)['authenticator_enabled'] is True
        ok('/api/authenticator/disable',{'password':PASSWORD,'code':codes[1]},token)
        assert ok('/api/me',token=token)['authenticator_enabled'] is False
        config=json.loads((DATA/'mail-private.json').read_text());config.update(enabled=True,host='127.0.0.1',port=smtp.server_address[1],security='starttls',**{'from':'Sternenepoche <test@example.org>'});(DATA/'mail-private.json').write_text(json.dumps(config))
        before=len(messages);ok('/api/registration/start',{'email':'tls-required@example.org'},status=503);assert len(messages)==before,'SMTP downgraded to plaintext'
        config['security']='local_test';(DATA/'mail-private.json').write_text(json.dumps(config));assert ok('/api/registration/status')['enabled'] is True
        ok('/api/register',{'name':'EmailBypass','password':PASSWORD},status=403)
        challenge=ok('/api/registration/start',{'email':'real-test@example.org'});email_code=last_email_code();assert 'code' not in challenge
        ok('/api/registration/start',{'email':'real-test@example.org'},status=429)
        wrong='000000' if email_code!='000000' else '111111';ok('/api/registration/verify',{'challenge_id':challenge['challenge_id'],'code':wrong},status=400)
        grant=ok('/api/registration/verify',{'challenge_id':challenge['challenge_id'],'code':email_code})
        ok('/api/registration/verify',{'challenge_id':challenge['challenge_id'],'code':email_code},status=400)
        body={'name':'EmailTester','password':PASSWORD,'registration_token':grant['registration_token'],'mode':'mensch','volk':'veyari','world_id':world}
        email_session=ok('/api/register',body);assert email_session['spieler'] is not None
        ok('/api/register',{**body,'name':'ReplayTester'},status=403)
        state=ok('/api/admin/status',token=key,admin=True);account=next(a for a in state['accounts'] if a['name']=='EmailTester');assert account['email']=='real-test@example.org' and account['email_verified_at']
        details=admin('player_details',id=account['id']);assert details['state'] and details['events']
        assert 'password' not in json.dumps(state) and encrypted not in json.dumps(state)
        for path in ['/api/admin/bootstrap','/data/online/spiel.sqlite3','/email-code-key.txt','/mail-private.json']:assert request(path)[0] in [401,404]
        if args.browser:browser_checks()
        print('PASS: disabled registration, five test accounts, independent RFC6238 TOTP, encrypted seeds, code replay protection, one-use recovery, restart and backup key, SMTP without world lock, TLS downgrade rejection, real local mail/code/profile flow, private registry and snapshots'+(', displayed QR decoded, browser TOTP/recovery download, mobile registration and private dashboard' if args.browser else '')+'.')
        print('Private isolated evidence:',DATA)
    finally:stop();smtp.shutdown();smtp.server_close()

if __name__=='__main__':main()
