import {setLesson,selectLab} from './labs.js';

const follow=document.querySelector('#follow-reading'), context=document.querySelector('#reading-context');
const selector=document.querySelector('#lab'), modes=document.querySelectorAll('[data-reading-mode]');
const stops=[...document.querySelectorAll('[data-lesson]')];
let following=true, active=null, scheduled=false;
let readingScroll=0, readingStop=null;
const article=document.querySelector('.reading-text');
const tail=document.createElement('div');tail.setAttribute('aria-hidden','true');article.append(tail);
function readingOffset(){
  const split=innerWidth<=760&&document.body.dataset.readingMode==='split';
  return document.querySelector('header').getBoundingClientRect().bottom+32+
    (split?document.querySelector('.example-workspace').getBoundingClientRect().height:0);
}
function findStop(){
  let next=stops[0];
  for(const stop of stops)if(stop.getBoundingClientRect().top<=readingOffset())next=stop;
  return next;
}
function ensureLastStop(){
  if(!article.getClientRects().length)return;
  // Supply only the trailing space needed to reach the final teaching heading.
  // Without it, short chapters and the last tour comparison can never activate.
  tail.style.height='0px';
  const last=stops.at(-1).getBoundingClientRect().top+scrollY;
  const missing=last-readingOffset()+innerHeight-document.documentElement.scrollHeight+24;
  const workspace=document.querySelector('.example-workspace');
  const style=getComputedStyle(workspace), pinned=style.position==='sticky';
  const panelHeight=parseFloat(style.maxHeight)||workspace.getBoundingClientRect().height;
  const stickyRoom=pinned?last-readingOffset()+24+parseFloat(style.top)+
    panelHeight-(article.getBoundingClientRect().bottom+scrollY):0;
  tail.style.height=Math.max(0,missing,stickyRoom)+'px';
}
function setFollowing(value){
  following=value;follow.setAttribute('aria-pressed',String(value));
  follow.textContent=value?'Following reading':'Follow reading';
  if(!value)context.textContent='Your comparison · scroll-follow paused';
}
function applyStop(next){
  active=next;setLesson(next.dataset.lesson,next.dataset.preset);
  context.textContent=next.dataset.label||next.querySelector('h2')?.textContent||next.textContent;
  for(const stop of stops)stop.classList.toggle('current-lesson',stop===active);
}
function update(){
  scheduled=false;if(!following||!document.querySelector('.reading-text').getClientRects().length)return;
  const next=findStop();
  if(!next||next===active)return;
  applyStop(next);
}
function schedule(){if(!scheduled){scheduled=true;requestAnimationFrame(update);}}
addEventListener('scroll',schedule,{passive:true});addEventListener('resize',()=>{ensureLastStop();schedule();});
document.addEventListener('rheon-manual-change',()=>setFollowing(false));
follow.addEventListener('click',()=>{
  setFollowing(!following);
  if(following){
    if(!article.getClientRects().length)applyStop(readingStop||active||stops[0]);
    else{active=null;update();}
  }
});
selector.addEventListener('change',()=>{
  // A deliberate lab selection has a shareable destination in the six-part tour.
  if(location.pathname.endsWith('/labs.html'))history.replaceState(null,'','#'+selector.value);
});
function readingMode(mode){
  const previous=document.body.dataset.readingMode;
  if(mode==='example'&&previous!=='example'){readingScroll=scrollY;readingStop=findStop();}
  document.body.dataset.readingMode=mode;
  for(const button of modes)button.setAttribute('aria-pressed',String(button.dataset.readingMode===mode));
  ensureLastStop();
  if(innerWidth<=760){
    if(mode==='example')window.scrollTo(0,0);
    else if(previous==='example')window.scrollTo(0,readingScroll);
  }
  schedule();
}
for(const button of modes)button.addEventListener('click',()=>readingMode(button.dataset.readingMode));
readingMode('text');setFollowing(true);
document.fonts.ready.then(()=>{ensureLastStop();schedule();});
// Hash/back navigation is evaluated only after the new scroll position settles.
addEventListener('hashchange',()=>{setFollowing(true);active=null;requestAnimationFrame(update);});
if(location.pathname.endsWith('/labs.html')&&location.hash){
  const lab=location.hash.slice(1),stop=stops.find(s=>s.id===lab);
  if(stop){stop.scrollIntoView();selectLab(lab);}
}
update();
