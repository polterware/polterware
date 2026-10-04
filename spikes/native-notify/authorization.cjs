const p=require('./probe.cjs'),fs=require('node:fs'),path=require('node:path');
(async()=>{
  p.kill();
  const bundleId=`com.polterware.sparky.spike2.authorization.${Date.now()}`;
  const app=path.join(p.root,'dist/prompt/Sparky.app');fs.mkdirSync(path.dirname(app),{recursive:true});
  fs.rmSync(app,{recursive:true,force:true});p.shell('/bin/cp',['-cR',path.join(p.root,'dist/napi/Sparky.app'),app]);
  p.shell('/usr/libexec/PlistBuddy',['-c',`Set :CFBundleIdentifier ${bundleId}`,path.join(app,'Contents/Info.plist')]);
  const config=path.join(app,'Contents/Resources/app/spike-config.json');fs.writeFileSync(config,JSON.stringify({form:'napi',data:path.join(p.root,'run/prompt')}));
  p.shell('/usr/bin/codesign',['--force','--deep','--sign','-','--identifier',bundleId,app]);
  const report={at:new Date().toISOString(),bundleId:bundleId,reason:'Fresh spike-only identity to observe a prompt without resetting the already-authorized spike2 or any real app'};
  report.status=await p.command('prompt','status');
  const pending=p.command('prompt','requestAuthorization',null,{timeoutMs:20000,show:true}).catch(e=>({error:e.message}));
  let ui;
  for(let i=0;i<3;i++){
    await p.wait(300);
    const pid=fs.readFileSync(path.join(p.dir('prompt'),'electron.pid'),'utf8');
    ui=JSON.parse(p.shell('/usr/bin/osascript',['-l','JavaScript',path.join(p.root,'authorization-ui.jxa'),pid]));
    if(ui.found)break;
  }
  report.ui=ui;report.authorization=await pending;
  p.kill(); // Close any timed-out authorization instance before the next command.
  report.permissionUnavailableApply=await p.command('prompt','apply',[p.item('permission-denied',60)]);
  report.pending=await p.command('prompt','pending');
  await p.command('prompt','removeAll','one:');
  fs.writeFileSync(path.join(p.root,'evidence/authorization.json'),JSON.stringify(report,null,2)+'\n');
  console.log(report);p.kill();
})().catch(e=>{console.error(e);p.kill();process.exitCode=1;});
