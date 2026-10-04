async (page) => {
 const phase = 'after';
 const root = 'plans/sparky-next/status/w3-polish/';
 const widths = [[800,560],[1280,800],[1920,1080]];
 const report = [];
 const snap = async name => {
   await page.screenshot({ path: `${root}${phase}-${name}-${page.viewportSize().width}.png`, animations: 'disabled', scale: 'css' });
   report.push({name, width:page.viewportSize().width, dialogs: await page.getByRole('dialog').count(), rows: await page.locator('tbody tr').count(), bodyOverflow: await page.evaluate(() => document.body.scrollWidth > innerWidth) });
 };
 const goto = async query => { await page.goto('http://127.0.0.1:5178/?'+query); await page.locator('.sparky-titlebar').waitFor(); await page.waitForTimeout(450); };
 for (const [width,height] of widths) {
   await page.setViewportSize({width,height});
   for (const route of ['today','inbox','minds','review','map','focus','search']) {
     await goto('route='+route);
     if (route==='map' && await page.getByRole('dialog').count()) { await page.getByRole('button',{name:'Keep tiles off'}).click(); }
     if (route==='search') { await page.getByRole('searchbox',{name:'Search memories'}).fill('milk'); await page.waitForTimeout(350); }
     await snap(route);
   }
   await goto('route=calendar');
   for (const view of ['Month','Week','Day']) { await page.getByRole('radiogroup',{name:'Calendar view'}).getByText(view,{exact:true}).click(); await page.waitForTimeout(300); await snap('calendar-'+view.toLowerCase()); }
   await goto('route=memory/memory-1'); await page.getByRole('textbox',{name:'Title',exact:true}).waitFor(); await snap('memory-detail');
   const schedule = page.getByRole('heading',{name:'Schedule',exact:true});
   if (await schedule.count()) { await schedule.scrollIntoViewIfNeeded(); } else { await page.getByRole('button',{name:'Save schedule'}).scrollIntoViewIfNeeded(); }
   await snap('schedule');
   await goto('route=today'); await page.getByRole('button',{name:'Open Commander'}).click(); await page.getByRole('textbox',{name:'Search or command'}).fill('milk'); await page.waitForTimeout(450); await snap('commander'); await page.keyboard.press('Escape');
   await page.getByRole('button',{name:'Settings',exact:true}).click();
   for (const label of ['General','Notifications','Focus','Calendar','Map','Search','Access','Activity','Data','Shortcuts','Advanced']) {
     await page.getByRole('navigation',{name:'Settings panels'}).getByRole('button',{name:label,exact:true}).click(); await page.waitForTimeout(150); await snap('settings-'+label.toLowerCase());
   }
   await goto('onboarding=true'); await page.getByRole('dialog').waitFor();
   for (let i=0;i<3;i++) { await snap('onboarding-'+(i+1)); if(i<2) await page.getByRole('button',{name:'Continue',exact:true}).click(); }
   for (const state of ['empty','loading','error','offline']) { await goto('route=inbox&state='+state); await snap('state-'+state); }
 }
 return report;
}
