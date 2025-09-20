import secrets from './secrets';

const username = "admin";
const { password } = secrets;

function sleep(time) {
    return new Promise(resolve => {
        setTimeout(resolve, time)
    })
}

export default {
    id: 'msbug',
    name: 'msbug',
    urlRegex: /^https:\/\/msbug-[0-9a-f]+\.ctfi\.ng/,
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

        await Promise.all([
            page.waitForNavigation(),
            /* Admin does not click random buttons */
            page.click('button#handle-button.bg-yellow-500.hover\\:bg-yellow-600.text-white.font-semibold.py-2.px-4.rounded.shadow')
        ]);

        await sleep(2000);
    },
}
