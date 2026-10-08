// Actual readiness drives the screen. No invented percentage or forced animation cycle.
(() => {
  'use strict';
  const screen=document.getElementById('sternen-loader');if(!screen)return;
  const status=screen.querySelector('[data-loader-status]'),pause=screen.querySelector('[data-loader-pause]');
  const image=screen.querySelector('img'),sources=[...screen.querySelectorAll('[data-loader-motion]')];
  const reduced=matchMedia('(prefers-reduced-motion: reduce)');
  let still=reduced.matches,started=performance.now(),finishTimer,fadeTimer,deadline,finished=false;
  try{still=still||localStorage.getItem('sternenepoche-loading-motion')==='paused';}catch{}
  let restore=[];
  function updateMotion(){
    sources.forEach(s=>s.media=still||document.hidden?'not all':'(prefers-reduced-motion: no-preference)');
    pause.textContent=still?'Animation starten':'Animation pausieren';pause.setAttribute('aria-pressed',String(still));
  }
  pause.addEventListener('click',()=>{still=!still;try{localStorage.setItem('sternenepoche-loading-motion',still?'paused':'running');}catch{}updateMotion();});
  reduced.addEventListener('change',e=>{still=e.matches;updateMotion();});
  document.addEventListener('visibilitychange',updateMotion);
  image.addEventListener('error',()=>{
    const webp=sources.find(s=>s.type==='image/webp');
    if(webp&&webp.media!=='not all'){webp.remove();sources.splice(sources.indexOf(webp),1);return;}
    sources.forEach(s=>s.remove());sources.length=0;
  });
  function release(){restore.forEach(([node,inert])=>{node.inert=inert;node.removeAttribute('aria-busy');});restore=[];}
  function hide(){
    if(finished)return;finished=true;clearTimeout(deadline);clearTimeout(finishTimer);
    sources.forEach(s=>s.media='not all');
    release();screen.classList.add('is-leaving');
    if(screen.contains(document.activeElement)){const main=document.querySelector('main');if(main){main.tabIndex=-1;main.focus({preventScroll:true});}}
    fadeTimer=setTimeout(()=>{screen.hidden=true;screen.classList.remove('is-leaving');},reduced.matches?0:400);
  }
  function ready(){clearTimeout(finishTimer);finishTimer=setTimeout(hide,Math.max(0,900-(performance.now()-started)));}
  function start(message){
    clearTimeout(finishTimer);clearTimeout(fadeTimer);clearTimeout(deadline);started=performance.now();finished=false;
    screen.hidden=false;screen.classList.remove('is-leaving');if(message)status.textContent=message;
    if(!restore.length)restore=[...document.querySelectorAll('main,header,footer')].map(node=>{const previous=node.inert;node.inert=true;node.setAttribute('aria-busy','true');return [node,previous];});
    updateMotion();deadline=setTimeout(hide,18000);
  }
  window.SternenLoading={start,ready,status:message=>{status.textContent=message;}};
  screen.querySelector('[data-loader-skip]').addEventListener('click',hide);
  window.addEventListener('pageshow',e=>{if(e.persisted)hide();});
  start();
  if(screen.dataset.kind!=='game'){
    const contentReady=()=>{
      const critical=[...document.querySelectorAll('.hero-art img')].map(img=>img.decode().catch(()=>{}));
      Promise.allSettled([document.fonts?.ready,...critical]).then(ready);
    };
    if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',contentReady,{once:true});else contentReady();
  }
})();
