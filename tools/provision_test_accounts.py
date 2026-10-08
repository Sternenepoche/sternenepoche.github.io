"""Create/reconcile the five requested accounts and a private local credentials file."""
from pathlib import Path
import argparse,getpass,json,os,re,secrets,subprocess,sys,urllib.request,urllib.error

ROOT=Path(__file__).resolve().parents[1]
NAMES=['LiveDemo','TestPilot2','TestPilot3','TestPilot4','TestPilot5']

def protect(path):
    if sys.platform=='win32':
        user=os.environ['USERDOMAIN']+'\\'+getpass.getuser()
        subprocess.run(['icacls',str(path),'/inheritance:r','/grant:r',user+':(F)','*S-1-5-18:(F)','*S-1-5-32-544:(F)'],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,creationflags=subprocess.CREATE_NO_WINDOW)
    else:path.chmod(0o600)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--data',type=Path,default=ROOT/'data/online')
    parser.add_argument('--game',default='http://127.0.0.1:8890')
    parser.add_argument('--admin',default='http://127.0.0.1:8891')
    parser.add_argument('--apply',action='store_true',help='Apply private account changes; LiveDemo keeps its realm but receives a test password if needed.')
    args=parser.parse_args()
    if not args.apply:parser.error('--apply is required to provision accounts')
    key=(args.data/'admin-token.txt').read_text().strip()
    def api(path,body=None,token=None,private=False):
        headers={'Content-Type':'application/json'}
        if private:headers['X-Sternenepoche-Admin']=key
        elif token:headers['Authorization']='Bearer '+token
        request=urllib.request.Request((args.admin if private else args.game)+path,data=json.dumps(body).encode() if body is not None else None,headers=headers)
        with urllib.request.urlopen(request,timeout=30) as r:return json.load(r)
    lobby=api('/api/lobby');world=lobby['world_id']
    existing=api('/api/admin/status',private=True)['accounts']
    assert not [a for a in existing if a['is_test'] and a['name'] not in NAMES],'Additional test accounts already exist; preserve them and resolve names manually.'
    path=args.data/'TESTKONTEN-ZUGAENGE.txt'
    if path.exists():
        protect(path)
        pairs=dict(re.findall(r'Testkonto-Name: ([^\r\n]+)\r?\nPasswort: ([^\r\n]+)',path.read_text(encoding='utf-8')))
        assert set(pairs)==set(NAMES),'Existing credentials document is incomplete; preserve and inspect it before retrying.'
    else:
        pairs={name:secrets.token_urlsafe(18) for name in NAMES}
        url=(args.data/'public-url.txt').read_text().strip() if (args.data/'public-url.txt').exists() else args.game
        fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600);os.close(fd);protect(path)
        text='STERNENEPOCHE - PRIVATE TESTZUGAENGE\nStand: 08.10.2026\n\nNur fuer den Betreiber und den jeweiligen Kontoinhaber. Diese Datei wird nicht veroeffentlicht.\n\n'
        text+=f'Spiel lokal: {args.game}\nSpiel online: {url}\nPrivates Dashboard: {args.admin}\nDatenbank: {args.data.resolve() / "spiel.sqlite3"}\n\n'
        text+='Fuenf Konten insgesamt: LiveDemo plus vier Testkonten. Keine E-Mail-Bestaetigung erforderlich.\nProvisionierung: Die Daten werden beim Ausfuehren abgeglichen; der Abschluss steht am Dateiende.\n\n'
        text+='\n\n'.join(f'Testkonto-Name: {name}\nPasswort: {password}' for name,password in pairs.items())
        text+='\n\nANMELDEN\nName und Passwort im Spiel eingeben, dann Anmelden. Bei TestPilot2 bis TestPilot5 anschliessend Volk und Spielweise waehlen und Platz belegen. LiveDemo behaelt sein bestehendes Reich; sein vorheriges Passwort und alte Sitzungen werden ersetzt.\n\nOPTIONALER AUTHENTICATOR\nSpielerprofil > Anmeldung mit Authenticator schuetzen > Passwort eingeben > QR-Code erzeugen. In Google Authenticator oder einer kompatiblen TOTP-App scannen und mit dem sechsstelligen App-Code bestaetigen. Acht Wiederherstellungscodes separat speichern. Beim spaeteren Login den aktuellen App-Code zusaetzlich eingeben. TOTP ist bei Erstellung fuer alle fuenf Konten ausgeschaltet.\n\nE-MAIL-REGISTRIERUNG\nVorbereitet, derzeit ausgeschaltet. SMTP spaeter in mail-private.json konfigurieren.\n\nBACKUP\nDatenbank und email-code-key.txt gemeinsam sichern. Dashboardbackups enthalten einen passenden .auth-key.txt-Begleitschluessel.\n'
        path.write_text(text,encoding='utf-8')
    api('/api/admin/action',{'action':'settings','world_id':world,'admission_limit':5},private=True)
    for name,password in pairs.items():
        try:session=api('/api/login',{'name':name,'password':password})
        except urllib.error.HTTPError as e:
            if e.code!=401:raise
            if next((a for a in existing if a['name']==name and a['authenticator_enabled']),None):raise RuntimeError('Existing Authenticator is enabled; preserve its protection and finish provisioning privately.') from None
            api('/api/admin/action',{'action':'test_account','world_id':world,'name':name,'password':password},private=True)
            session=api('/api/login',{'name':name,'password':password})
        else:api('/api/admin/action',{'action':'test_account','world_id':world,'name':name},private=True)
        me=api('/api/me',token=session['token']);assert me['name']==name
        api('/api/logout',{},session['token'])
    status=api('/api/admin/status',private=True)
    assert status['lobby']['world_id']==world and status['lobby']['freigegebene_plaetze']==5
    assert len([a for a in status['accounts'] if a['is_test']])==5
    for name in ['email-code-key.txt','mail-private.json','admin-token.txt']:
        protect(args.data/name)
    for keyfile in (args.data/'backups').glob('*.auth-key.txt'):protect(keyfile)
    text=path.read_text(encoding='utf-8')
    if 'ABGESCHLOSSEN:' not in text:
        with path.open('a',encoding='utf-8') as f:f.write('\nABGESCHLOSSEN: Alle fuenf Logins erfolgreich geprueft, fuenf Spielerplaetze freigegeben, bestehende Welt erhalten.\n')
    print('PASS: five private credentials saved, every login verified, admission limit five, existing world retained; no credentials printed.')
    print('Local document:',path.resolve())

if __name__=='__main__':main()
