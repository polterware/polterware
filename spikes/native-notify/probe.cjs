// Bounded LaunchServices probes. Never signal a process outside this spike's dist.
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const root = __dirname;
const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
function shell(file, args) {
  const p = spawnSync(file, args, { encoding:'utf8', timeout:15000 });
  if (p.error || p.status) throw Error(p.error?.message || p.stderr);
  return p.stdout.trim();
}
function processes() {
  return shell('/bin/ps', ['-axo','pid=,rss=,comm=']).split('\n').flatMap(line => {
    const match = line.trim().match(/^(\d+)\s+(\d+)\s+(.+)$/);
    return match && match[3].startsWith(path.join(root,'dist') + '/') ? [{pid:+match[1],rssKiB:+match[2],executable:match[3]}] : [];
  });
}
function kill() {
  const found = processes();
  for (const p of found) { try { process.kill(p.pid, 'SIGKILL'); } catch (e) { if (e.code !== 'ESRCH') throw e; } }
  return found;
}
const dir = form => path.join(root,'run',form,'run');
async function command(form, method, payload=null, options={}) {
  fs.mkdirSync(dir(form),{recursive:true});
  const result = path.join(dir(form),'result.json');
  fs.rmSync(result,{force:true});
  fs.writeFileSync(path.join(dir(form),'command.json'), JSON.stringify(payload));
  shell('/usr/bin/open', ['-n','-a',path.join(root,'dist',form,'Sparky.app'),'--args',`--command=${method}`, ...(options.hold ? ['--hold'] : []), ...(options.show ? ['--show-window'] : [])]);
  const deadline = Date.now() + (options.timeoutMs || 15000);
  while (Date.now() < deadline) {
    if (fs.existsSync(result)) {
      const value = JSON.parse(fs.readFileSync(result));
      if (value.command === method) {
        await wait(300);
        return value;
      }
    }
    await wait(100);
  }
  throw Error(`${form} ${method}: bounded result wait expired`);
}
function item(id, delay=20, repeat=null) {
  return { id:`one:${id}`, memoryId:'01ARZ3NDEKTSV4RRFFQ69G5FAV', title:`W0 ${id}`, body:'Synthetic native notification spike',
    category:'reminder', fireAt:new Date(Date.now()+delay*1000).toISOString(), repeat };
}
function logs(form) {
  const file = path.join(dir(form),'notify.jsonl');
  return fs.existsSync(file) ? fs.readFileSync(file,'utf8').trim().split('\n').filter(Boolean).map(JSON.parse) : [];
}
module.exports = {shell,command,kill,processes,item,wait,dir,logs,root};
if (require.main === module) (async () => {
  const [form,method,raw] = process.argv.slice(2);
  console.log(JSON.stringify(method === 'kill' ? kill() : await command(form,method,raw ? JSON.parse(raw) : null),null,2));
})().catch(e=>{ console.error(e.message); process.exitCode=1; });
