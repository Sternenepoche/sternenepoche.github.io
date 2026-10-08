"""Render the seven-layer neural star as a real high-resolution GIF.

The ImageGen original supplies the metal. Canvas separates its four panels,
animates the network and traces the edge; FFmpeg encodes the seamless loop.
"""
from pathlib import Path
import argparse, base64, json, subprocess, shutil, sys
from playwright.sync_api import sync_playwright

ROOT=Path(__file__).resolve().parents[1]
ART=ROOT/'docs/branding'
RUN=ROOT/'laeufe/website-2026-10-08'

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--preview',action='store_true')
    parser.add_argument('--size',type=int,default=1280)
    parser.add_argument('--fps',type=int,default=25)
    args=parser.parse_args()
    frames=RUN/'neuralstern-frames';frames.mkdir(parents=True,exist_ok=True)
    source='data:image/png;base64,'+base64.b64encode((ART/'sternenepoche-sternentor-original.png').read_bytes()).decode()
    animation=(ROOT/'tools/brand_animation.js').read_text(encoding='utf-8')
    html='<html><body style="margin:0;background:#050b14;display:grid;place-items:center;height:100vh"><canvas style="width:min(96vw,96vh);height:auto"></canvas><script>window.SOURCE='+json.dumps(source)+';window.RENDER_SIZE='+str(args.size)+';</script><script>'+animation+'</script>'
    live='<script>window.ready.then(()=>{const begin=performance.now();function loop(now){brandAnimation.drawFrame((now-begin)/1000%18*25,25);requestAnimationFrame(loop);}requestAnimationFrame(loop);});</script></body></html>'
    (RUN/'neuralstern-preview.html').write_text(html+live,encoding='utf-8')
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome' if sys.platform=='win32' else None,headless=True)
        page=browser.new_page(viewport={'width':args.size,'height':args.size})
        page.set_content(html+'</body></html>');page.evaluate('window.ready')
        indices=[0,25,65,113,140,170,222,283,315,345,390,419,449] if args.preview else range(18*args.fps)
        for i in indices:
            data=page.evaluate('([index,fps])=>brandAnimation.drawFrame(index,fps)',[i,args.fps])
            (frames/f'{i:04d}.png').write_bytes(base64.b64decode(data))
        browser.close()
    if args.preview:
        print('Preview frames: '+str(frames));return
    gif=ART/f'sternenepoche-neuralstern-{args.size}.gif'
    ffmpeg=shutil.which('ffmpeg') or r'C:\ffmpeg\bin\ffmpeg.exe'
    subprocess.run([ffmpeg,'-hide_banner','-loglevel','error','-y','-framerate',str(args.fps),
        '-i',str(frames/'%04d.png'),'-filter_complex',
        '[0:v]split[a][b];[a]palettegen=stats_mode=full:reserve_transparent=0[p];[b][p]paletteuse=dither=bayer:bayer_scale=3',
        '-loop','0',str(gif)],check=True)
    subprocess.run([ffmpeg,'-hide_banner','-loglevel','error','-y','-i',str(gif),
        '-vf','scale=384:384','-loop','0','-c:v','libwebp_anim','-quality','82',
        str(ART/'sternenepoche-neuralstern-384.webp')],check=True)
    subprocess.run([ffmpeg,'-hide_banner','-loglevel','error','-y','-i',str(gif),
        '-filter_complex',
        '[0:v]scale=384:384,split[a][b];[a]palettegen=stats_mode=full:reserve_transparent=0[p];[b][p]paletteuse=dither=bayer:bayer_scale=3',
        '-loop','0',str(ART/'sternenepoche-neuralstern-384.gif')],check=True)
    brand=ROOT/'web-client/brand';brand.mkdir(exist_ok=True)
    subprocess.run([ffmpeg,'-hide_banner','-loglevel','error','-y','-framerate',str(args.fps),'-i',str(frames/'%04d.png'),
        '-vf','scale=768:768','-loop','0','-c:v','libwebp_anim','-quality','86',str(brand/'neuralstern-768.webp')],check=True)
    subprocess.run([ffmpeg,'-hide_banner','-loglevel','error','-y','-i',str(gif),'-filter_complex',
        '[0:v]scale=768:768,split[a][b];[a]palettegen=stats_mode=full:reserve_transparent=0[p];[b][p]paletteuse=dither=bayer:bayer_scale=3',
        '-loop','0',str(brand/'neuralstern-768.gif')],check=True)
    subprocess.run([ffmpeg,'-hide_banner','-loglevel','error','-y','-i',str(frames/'0000.png'),'-vf','scale=768:768',
        '-q:v','3','-frames:v','1',str(brand/'neuralstern-poster.jpg')],check=True)
    info={'size':[args.size,args.size],'fps':args.fps,'frames':18*args.fps,'duration_seconds':18,'loop':'infinite',
          'file':gif.name,'bytes':gif.stat().st_size,'layers':[4,6,8,8,6,4,3],'signal_passes':3,
          'sequence':['pulse','four panels open','seven-layer neural core transmits three times','panels close','green signal follows star edge','signal enters core','rest pose'],
          'source':'Preserved ImageGen stellar aperture; deterministic Canvas choreography'}
    (ART/'animation.json').write_text(json.dumps(info,indent=2),encoding='utf-8')
    print(json.dumps(info))

if __name__=='__main__':main()
