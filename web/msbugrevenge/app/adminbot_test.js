/*
 * Similar implementation to the real admin bot for testing. You only need to submit the URL to your instance.
 * The remote adminbot will validate this URL, for local use its hardcoded to 127.0.0.1:5000 below.
 *
 * npm i puppeteer
 *
 */

const puppeteer = require("puppeteer");

const username = "admin";
const password = "password";

function sleep(time) {
  return new Promise(resolve => {
    setTimeout(resolve, time)
  })
}

const visit = async (url) => {
  let browser;
  try {
    browser = await puppeteer.launch({
      headless: true,
      pipe: true,
      args: [
        "--no-sandbox",
        "--disable-setuid-sandbox",
        "--js-flags=--jitless",
        "--log-level=3"
      ],
      dumpio: true
    });

    const ctx = await browser.createBrowserContext();

    const page = await ctx.newPage();

    const { origin } = new URL(url);

    /* Bot logs in */
    await page.goto(`${origin}/login`, { waitUntil: 'load' });

    await page.type("input[name=username]", username);
    await page.type("input[name=password]", password);
    await Promise.all([
      page.waitForNavigation(),
      page.click("button[type=submit]")
    ]);

    /* Bot goes to admin page */
    await page.goto(`${origin}/admin`, { timeout: 5000, waitUntil: 'networkidle0' });

    /* Bot types password again to confirm its him */
    await page.waitForSelector('textarea[id^="password-"]');
    await page.type('textarea[id^="password-"]', password);
        
    /* Bot clicks on the handle button. He is smart so he only clicks the actual button. */
    await sleep(5000);
    await page.click('button#handle-button.bg-yellow-500.hover\\:bg-yellow-600.text-white.font-semibold.py-2.px-4.rounded.shadow')

    await sleep(5000);

  } catch (error) {
    console.log(error);
  }
  finally {
    if (browser) await browser.close();
  }
}

visit("http://127.0.0.1:5000/");
