// Reproducible live acceptance; emits synthetic records, never core credentials.
const p=require('./probe.cjs');
const fs=require('node:fs');
const path=require('node:path');
const evidence=path.join(p.root,'evidence','runtime.json');
const report={at:new Date().toISOString(),os:p.shell('/usr/bin/sw_vers',[]),forms:{}};
function save(){fs.writeFileSync(evidence,JSON.stringify(report,null,2)+'\n');}
function ps(){return p.processes().map(x=>({...x,executable:path.relative(p.root,x.executable)}));}
function ui(title,action){return JSON.parse(p.shell('/usr/bin/osascript',['-l','JavaScript',path.join(p.root,'notification-ui.jxa'),title,...(action?[action]:[])]));}
async function core(form,route,body){
  const state=JSON.parse(fs.readFileSync(path.join(p.dir(form),'core.json')));
  const res=await fetch(new URL(route,state.endpoint),{method:body?'POST':'GET',headers:{Authorization:`Bearer ${state.token}`,'Content-Type':'application/json'},body:body?JSON.stringify(body):undefined,signal:AbortSignal.timeout(2000)});
  if(!res.ok)throw Error(`stub HTTP ${res.status}`);return res.json();
}
const frontmost=()=>p.shell('/usr/bin/osascript',['-e','tell application "System Events" to get bundle identifier of first application process whose frontmost is true']);
async function round(form){
  const r=report.forms[form]={};save();p.kill();
  fs.rmSync(path.join(p.dir(form),'plan.json'),{force:true});
  r.status=await p.command(form,'status');
  await p.command(form,'removeAll','one:'); await p.command(form,'removeAll','rem:');
  const one=p.item(`${form}-survival`,8), rep=p.item(`${form}-repeat`,13,{tz:'UTC',second:new Date(Date.now()+13000).getUTCSeconds()});
  r.arm=await p.command(form,'apply',[one,rep]);
  r.identicalPlan=await p.command(form,'apply',[one,rep],{hold:true});
  await core(form,'/plan',{items:[one,rep]});
  r.preKill=ps();r.killed=p.kill().map(x=>({...x,executable:path.relative(p.root,x.executable)}));r.killedAt=new Date().toISOString();save();
  await p.wait(14500);r.afterDeadlineProcesses=ps();
  r.delivered=await p.command(form,'delivered');r.pending=await p.command(form,'pending');save();
  console.log(form,'survival',JSON.stringify(r.delivered));
  await p.command(form,'removeAll','one:');p.kill();
  r.actions=[];
  for(const action of ['Done','Snooze']){
    fs.rmSync(path.join(p.dir(form),'plan.json'),{force:true});
    const start=p.logs(form).length;const item=p.item(`${form}-${action.toLowerCase()}`,6);
    const a={action,item,arm:await p.command(form,'apply',[item],{hold:true})};
    await core(form,'/plan',{items:[item]});
    a.killed=p.kill().map(x=>({...x,executable:path.relative(p.root,x.executable)}));a.closedAt=new Date().toISOString();
    await p.wait(5900);a.processesBeforeClick=ps();a.ui=ui(item.title,action);
    const deadline=Date.now()+15000;
    while(Date.now()<deadline){
      const events=p.logs(form).slice(start);
      if(events.some(e=>e.event==='idle-exit')){a.logs=events;break;}
      await p.wait(200);
    }
    a.logs ||= p.logs(form).slice(start);
    const applied=a.logs.find(e=>e.event==='action-applied');
    if(applied && a.ui.clicked)a.clickToAppliedMs=Date.parse(applied.at)-a.ui.clickAtMs;
    a.after=ps();a.coreEvents=await core(form,'/events');a.plan=await core(form,'/plan');
    r.actions.push(a);save();console.log(form,action,JSON.stringify({ui:a.ui,clickToAppliedMs:a.clickToAppliedMs,plan:a.plan,logs:a.logs}));
    p.kill();await p.command(form,'removeAll','one:');
  }
  const start=p.logs(form).length;
  r.hooks=await p.command(form,'testHooks');await p.wait(1500);r.hookLogs=p.logs(form).slice(start);save();p.kill();
  fs.writeFileSync(path.join(p.dir(form),'plan.json'),'[]');
  r.hidden={beforeFrontmost:frontmost(),startedAt:new Date().toISOString()};
  const idx=p.logs(form).length;
  p.shell('/usr/bin/open',['-g','-j','-a',path.join(p.root,'dist',form,'Sparky.app'),'--args','--background-replan']);
  r.hidden.samples=[];
  for(let i=0;i<13;i++){await p.wait(1000);r.hidden.samples.push({at:new Date().toISOString(),frontmost:frontmost(),processes:ps()});}
  r.hidden.logs=p.logs(form).slice(idx);r.hidden.after=ps();
  r.bundleAllocatedKiB=+p.shell('/usr/bin/du',['-sk',path.join(p.root,'dist',form,'Sparky.app')]).split(/\s/)[0];
  function size(f){const s=fs.lstatSync(f);return s.isDirectory()?fs.readdirSync(f).reduce((n,x)=>n+size(path.join(f,x)),0):s.size;}
  r.bundleLogicalBytes=size(path.join(p.root,'dist',form,'Sparky.app'));
  r.moduleBytes=fs.statSync(path.join(p.root,'dist',form,'Sparky.app','Contents/Resources/app/native',form==='napi'?'notify.node':'libNotify.dylib')).size;
  r.checks={
    authorized:r.status.value?.authorization==='authorized',
    idempotentDiff:r.identicalPlan.value?.unchanged===2,
    killedElectronAndCore:r.killed.some(x=>x.executable.endsWith('/MacOS/Electron'))&&r.killed.some(x=>x.executable.endsWith('/stubcore')),
    noProcessesAtDelivery:r.afterDeadlineProcesses.length===0,
    oneShotDelivered:r.delivered.value?.some(x=>x.id===one.id&&!x.repeats),
    repeatDelivered:r.delivered.value?.some(x=>x.id===rep.id&&x.repeats&&x.trigger==='UNCalendarNotificationTrigger'),
    repeatStillPending:r.pending.value?.some(x=>x.id===rep.id&&x.repeats),
    coldActions:r.actions.every(a=>a.ui.clicked&&a.processesBeforeClick.length===0&&a.clickToAppliedMs>=0&&a.coreEvents.events.some(e=>e.kind===a.action.toLowerCase().replace('done','complete'))),
    doneState:r.actions[0].plan.items.length===0,
    snoozeState:r.actions[1].plan.items.some(x=>x.id.endsWith(':snooze')),
    actionIdleExit:r.actions.every(a=>a.logs.some(e=>e.event==='idle-exit'&&e.windows===0)&&!a.after.some(x=>x.executable.endsWith('/MacOS/Electron'))),
    hooks:['wake','timezone'].every(reason=>r.hookLogs.some(e=>e.event==='native-event'&&e.reason===reason))&&r.hookLogs.some(e=>e.event==='replan'),
    hiddenFrontmost:r.hidden.samples.every(s=>s.frontmost===r.hidden.beforeFrontmost)&&!r.hidden.beforeFrontmost.startsWith('com.polterware.sparky'),
    hiddenNoDock:r.hidden.logs.some(e=>e.event==='ready'&&e.dockVisible===false&&e.windows===0),
    hiddenReplanAndExit:r.hidden.logs.some(e=>e.event==='replan'&&e.windows===0)&&r.hidden.logs.some(e=>e.event==='idle-exit'&&e.windows===0),
    noRuntimeErrors:![...r.actions.flatMap(a=>a.logs),...r.hookLogs,...r.hidden.logs].some(e=>e.event.endsWith('error'))
  };
  save();p.kill();console.log(form,'hidden and metrics saved',JSON.stringify(r.checks));
}
(async()=>{
  report.originalFrontmost=frontmost();
  for(const form of ['napi','swift'])await round(form);
  p.kill();
  p.shell('/usr/bin/osascript',['-e',`tell application id "${report.originalFrontmost}" to activate`]);
  report.finishedAt=new Date().toISOString();report.remaining=ps();report.allPassed=Object.values(report.forms).every(r=>Object.values(r.checks).every(Boolean));save();if(!report.allPassed)process.exitCode=1;
})().catch(e=>{report.error=e.stack;save();console.error(e);p.kill();if(report.originalFrontmost)p.shell('/usr/bin/osascript',['-e',`tell application id "${report.originalFrontmost}" to activate`]);process.exitCode=1;});
