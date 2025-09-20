const puppeteer = require("puppeteer");

const ADMIN_PASSWORD = process.env.ADMIN_PASSWORD || "test_password";
const sleep = (ms) => new Promise(r => setTimeout(r, ms));

const path = require("path");
const ext = path.resolve(__dirname, "./extension/");

const visit = async (url) => {
  let browser;
  try {
    browser = await puppeteer.launch({
      headless: "new",
      pipe: true,
      args: [
        "--no-sandbox",
        "--disable-setuid-sandbox",
        `--disable-extensions-except=${ext}`,
        `--load-extension=${ext}`
      ],
      dumpio: true
    });
    
    const page = await browser.newPage();
    
    await page.goto("https://[VICTIM SITE]/login", { timeout: 5000, waitUntil: 'networkidle2' });
    
    await page.waitForSelector('input[id="pass"]', {timeout: 5000});

    await page.type('input[id="user"]', "fizzbuzz101");
    await page.type('input[id="pass"]', ADMIN_PASSWORD);
    await page.click('button[type="submit"]');

    await sleep(3000);
    
    // go to exploit page
    await page.goto(url, { timeout: 300_000, waitUntil: 'networkidle2' });
    
    // this sleep is extra long to make your life easier, you don't actually need the full 5 minutes to solve this challenge
    // this also means that you shouldn't spam submit the admin bot, since you're just going to OOM yourself
    await sleep(300_000);

    await browser.close();
    browser = null;
  } catch (err) {
    console.log(err);
  } finally {
    if (browser) await browser.close();
  }
};

module.exports = { visit };
