import flag from './flag.txt';

function sleep(time) {
  return new Promise(resolve => {
    setTimeout(resolve, time)
  })
}

export default {
  name: 'safe-url admin bot',
  timeout: 15_000,
  handler: async (url, ctx) => {
    let page = await ctx.newPage();
    await page.goto("https://safe-url.ctfi.ng", { timeout: 3000, waitUntil: 'domcontentloaded' });

    await page.evaluate((flag) => {
        localStorage.setItem("flag", flag);
    }, flag);

    await sleep(1000);
    await page.close();

    page = await ctx.newPage();
    await page.goto(url, { timeout: 3000, waitUntil: 'domcontentloaded' });
    await sleep(5000);
  }
}