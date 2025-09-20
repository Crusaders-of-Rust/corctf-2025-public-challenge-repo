/*
 * Similar implementation to the real admin bot for testing. Provide the URL to your submitted translation.
 * The remote adminbot will validate this URL, for local use its hardcoded to 127.0.0.1:8080 below.
 *
 * npm i puppeteer
 *
 */
const puppeteer = require("puppeteer");

const TRANSLATION_ID = 1 // CHANGE THIS
const TRANSLATION_URL = "http://127.0.0.1:8080/admin/translation?id=" + TRANSLATION_ID

const username = "admin";
const password = "admin";

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

        /* Admin logs in */
        await page.goto(`${origin}/login`, { waitUntil: 'load' });

        await page.type("input[name=username]", username);
        await page.type("input[name=password]", password);
        await Promise.all([
            page.waitForNavigation(),
            page.click("button[type=submit]")
        ]);

        /* Checks your translation */
        await page.goto(url, { timeout: 5000, waitUntil: 'networkidle0' });

        /* Rejects it as it's probably shit */
        await Promise.all([
            page.waitForNavigation(),
            page.click("button[name=reject]")
        ]);

        await sleep(2000);
        
      } catch (error) {
        console.log(error);
      }
      finally {
        if (browser) await browser.close();
      }
}

visit(TRANSLATION_URL);