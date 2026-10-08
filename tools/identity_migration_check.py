"""Verify additive account migration on an isolated SQLite backup of an existing world."""
from pathlib import Path
import argparse,json,shutil,sqlite3,subprocess,sys,time,urllib.request,uuid

ROOT=Path(__file__).resolve().parents[1]

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--server-exe',type=Path,required=True)
    parser.add_argument('--data',type=Path,default=ROOT/'data/online')
    args=parser.parse_args()
    out=ROOT/'laeufe/identity-2026-10-08'/('migration-'+uuid.uuid4().hex[:8]);out.mkdir(parents=True)
    with sqlite3.connect(args.data/'spiel.sqlite3') as source,sqlite3.connect(out/'spiel.sqlite3') as copy:
        source.backup(copy)
        before=copy.execute('select id,name,password,sid,mode,banned from accounts order by id').fetchall()
        sessions=copy.execute('select * from sessions order by hash').fetchall()
        assert before,'Use an existing database with at least one account'
    for name in ['email-code-key.txt','mail-private.json']:
        if (args.data/name).exists():shutil.copyfile(args.data/name,out/name)
    def api(path,token=None):
        request=urllib.request.Request('http://127.0.0.1:18989'+path,headers={'X-Sternenepoche-Admin':token} if token else {})
        with urllib.request.urlopen(request,timeout=10) as r:return json.load(r)
    process=subprocess.Popen([str(args.server_exe),'--data',str(out),'--bind','127.0.0.1:18988','--admin-bind','127.0.0.1:18989'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,creationflags=subprocess.CREATE_NO_WINDOW if sys.platform=='win32' else 0)
    try:
        for _ in range(100):
            if process.poll() is not None:raise AssertionError('Migrated server failed to start')
            try:key=api('/api/admin/bootstrap')['key'];break
            except OSError:time.sleep(.1)
        else:raise AssertionError('Migrated server did not start')
        status=api('/api/admin/status',key)
        with sqlite3.connect(out/'spiel.sqlite3') as db:
            assert db.execute('select id,name,password,sid,mode,banned from accounts order by id').fetchall()==before
            assert db.execute('select * from sessions order by hash').fetchall()==sessions
            assert db.execute('select count(*) from registered_players').fetchone()[0]==len(before)
            assert db.execute('select count(*) from player_states').fetchone()[0]==len(before)
            runtime=json.loads(db.execute('select runtime from checkpoint').fetchone()[0])
            assert status['lobby']['world_id']==runtime['world_id']
            assert {a['name']:a['spieler'] for a in status['accounts']}=={a[1]:a[3] for a in before}
        print('PASS: existing account IDs, Argon2 hashes, realm ownership, play modes, bans and sessions preserved; readable registry and states added in isolated copy.')
        print('Private migration evidence:',out)
    finally:process.terminate();process.wait(timeout=15)

if __name__=='__main__':main()
