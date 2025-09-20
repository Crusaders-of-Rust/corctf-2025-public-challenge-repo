// npm i puppeteer
// script to emulate admin bot
const puppeteer = require("puppeteer");

const FLAG = "corctf{test_flag}";
const SITE = "https://safe-url.ctfi.ng";

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
            ],
            dumpio: true
        });

        // incognito btw
        const ctx = await browser.createBrowserContext();

        let page = await ctx.newPage();
        await page.goto(SITE, { timeout: 3000, waitUntil: 'domcontentloaded' });

        await page.evaluate((flag) => {
            localStorage.setItem("flag", flag);
        }, FLAG);

        await sleep(1000);
        await page.close();

        page = await ctx.newPage();
        await page.goto(url, { timeout: 3000, waitUntil: 'domcontentloaded' });
        await sleep(5000);

        await browser.close();
        browser = null;
    } catch (err) {
        console.log(err);
    } finally {
        if (browser) await browser.close();
    }
};

visit("EXPLOIT_URL");