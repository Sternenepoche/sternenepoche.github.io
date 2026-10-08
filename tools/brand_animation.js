// Export harness for the same renderer used by the live loading screen.
const source=new Image();source.src=window.SOURCE;
const renderer=createNeuralStar(document.querySelector('canvas'),source,window.RENDER_SIZE||1280);
window.ready=renderer.ready;
window.brandAnimation={duration:renderer.duration,layers:renderer.layers,drawFrame(index,fps=25){
 renderer.draw(index/fps);
 return document.querySelector('canvas').toDataURL('image/png').split(',')[1];
}};
