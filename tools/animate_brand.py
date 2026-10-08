"""Render the neural star as a compact downloadable GIF.

The ImageGen original supplies the metal. Canvas separates its four panels,
animates the network and traces the edge; FFmpeg encodes the seamless loop.
"""
from pathlib import Path
import argparse, base64, json, subprocess, shutil, sys
from playwright.sync_api import sync_playwright
from PIL import Image

ROOT=Path(__file__).resolve().parents[1]
ART=ROOT/'docs/branding'
RUN=ROOT/'laeufe/website-2026-10-08'

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--preview',action='store_true')
    parser.add_argument('--size',type=int,default=512)
    parser.add_argument('--fps',type=int,default=12)
    args=parser.parse_args()
    frames=RUN/f'neuralstern-frames-{args.size}-{args.fps}';frames.mkdir(parents=True,exist_ok=True)
    source='data:image/png;base64,'+base64.b64encode((ART/'sternenepoche-sternentor-original.png').read_bytes()).decode()
    animation=(ROOT/'web-client/brand-animation.js').read_text(encoding='utf-8')+'\n'+(ROOT/'tools/brand_animation.js').read_text(encoding='utf-8')
    html='<html><body style="margin:0;background:#050b14;display:grid;place-items:center;height:100vh"><canvas style="width:min(96vw,96vh);height:auto"></canvas><script>window.SOURCE='+json.dumps(source)+';window.RENDER_SIZE='+str(args.size)+';</script><script>'+animation+'</script>'
    live='<script>window.ready.then(()=>{const begin=performance.now();function loop(now){brandAnimation.drawFrame((now-begin)/1000%18*25,25);requestAnimationFrame(loop);}requestAnimationFrame(loop);});</script></body></html>'
    (RUN/'neuralstern-preview.html').write_text(html+live,encoding='utf-8')
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome' if sys.platform=='win32' else None,headless=True)
        page=browser.new_page(viewport={'width':args.size,'height':args.size})
        page.set_content(html+'</body></html>');page.evaluate('window.ready')
        indices=[round(t*args.fps) for t in [0,1,2.6,4.5,5.6,6.8,8.9,11.3,12.6,13.8,15.6,16.8,17.9]] if args.preview else range(18*args.fps)
        for i in indices:
            data=page.evaluate('([index,fps])=>brandAnimation.drawFrame(index,fps)',[i,args.fps])
            (frames/f'{i:04d}.png').write_bytes(base64.b64decode(data))
        browser.close()
    if args.preview:
        print('Preview frames: '+str(frames));return
    gif=ART/f'sternenepoche-neuralstern-{args.size}.gif'
    ffmpeg=shutil.which('ffmpeg') or r'C:\ffmpeg\bin\ffmpeg.exe'
    raw=RUN/'neuralstern-before-optimization.gif'
    subprocess.run([ffmpeg,'-hide_banner','-loglevel','error','-y','-framerate',str(args.fps),
        '-i',str(frames/'%04d.png'),'-filter_complex',
        '[0:v]split[a][b];[a]palettegen=max_colors=96:stats_mode=diff:reserve_transparent=1[p];[b][p]paletteuse=dither=none:diff_mode=rectangle',
        '-loop','0',str(raw)],check=True)
    gifsicle=shutil.which('gifsicle') or str(RUN/'gifsicle-tool/gifsicle.exe')
    subprocess.run([gifsicle,'-O3','--lossy=80',str(raw),'-o',str(gif)],check=True)
    small_raw=RUN/'neuralstern-384-before-optimization.gif'
    subprocess.run([ffmpeg,'-hide_banner','-loglevel','error','-y','-framerate',str(args.fps),'-i',str(frames/'%04d.png'),
        '-filter_complex','[0:v]scale=384:384:flags=lanczos,split[a][b];[a]palettegen=max_colors=80:stats_mode=diff:reserve_transparent=1[p];[b][p]paletteuse=dither=none:diff_mode=rectangle',
        '-loop','0',str(small_raw)],check=True)
    subprocess.run([gifsicle,'-O3','--lossy=80',str(small_raw),'-o',str(ART/'sternenepoche-neuralstern-384.gif')],check=True)
    subprocess.run([ffmpeg,'-hide_banner','-loglevel','error','-y','-framerate',str(args.fps),'-i',str(frames/'%04d.png'),
        '-vf','scale=384:384:flags=lanczos','-loop','0','-c:v','libwebp_anim','-quality','72','-compression_level','6',
        str(ART/'sternenepoche-neuralstern-384.webp')],check=True)
    with Image.open(gif) as optimized: actual_frames=optimized.n_frames
    info={'size':[args.size,args.size],'fps':args.fps,'frames':actual_frames,'rendered_frames':18*args.fps,'duration_seconds':18,'loop':'infinite',
          'file':gif.name,'bytes':gif.stat().st_size,'layers':[4,6,8,8,6,4,3],'signal_passes':3,
          'sequence':['pulse','four panels open','seven-layer neural core transmits three times','panels close','green signal follows star edge','signal enters core','rest pose'],
          'source':'Preserved ImageGen stellar aperture; shared live Canvas renderer',
          'legacy_url':'sternenepoche-neuralstern-1280.gif',
          'legacy_url_serves':'same compact 512px GIF; original master is no longer published',
          'web_loading':'Canvas up to 1280px; small WebP texture with PNG fallback, static JPEG for reduced motion'}
    if args.size==512 and gif.stat().st_size>1100000: raise ValueError('Public GIF exceeded its 1.1 MB budget')
    metadata=ART/'animation.json' if args.size==512 else RUN/f'animation-{args.size}.json'
    metadata.write_text(json.dumps(info,indent=2),encoding='utf-8')
    print(json.dumps(info))

if __name__=='__main__':main()
