import secrets from './secrets';

const username = "admin";
const { password } = secrets;

function sleep(time) {
    return new Promise(resolve => {
        setTimeout(resolve, time)
    })
}

export default {
    id: 'msbugrevenge',
    name: 'msbug\'s revenge',
    urlRegex: /^https:\/\/msbugrevenge-[0-9a-f]+\.ctfi\.ng\//,
    timeout: 10000,
    handler: async (url, ctx) => {
        const page = await ctx.newPage();

        const { origin } = new URL(url);
        await page.goto(`${origin}/login`, { waitUntil: 'load' });

        await page.type("input[name=username]", username);
        await page.type("input[name=password]", password);
        await Promise.all([
            page.waitForNavigation(),
            page.click("button[type=submit]")
        ]);

        await page.goto(`${origin}/admin`, { timeout: 5000, waitUntil: 'networkidle0' });

        /* Bot types password again to confirm its him */
        await page.waitForSelector('textarea[id^="password-"]');
        await page.type('textarea[id^="password-"]', password);

        /* Bot clicks on the handle button. He is smart so he only clicks the actual button. */
        await sleep(5000);
        await page.click('button#handle-button.bg-yellow-500.hover\\:bg-yellow-600.text-white.font-semibold.py-2.px-4.rounded.shadow')

        await sleep(5000);
    },
}
