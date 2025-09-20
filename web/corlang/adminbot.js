import secrets from './secrets';

const username = "admin";
const { password } = secrets;

function sleep(time) {
    return new Promise(resolve => {
        setTimeout(resolve, time)
    })
}

export default {
    id: 'corlang',
    name: 'corlang',
    urlRegex: /^https:\/\/corlang-[0-9a-f]+\.ctfi\.ng\/admin\/translation\?id=\d+$/,
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

        await page.goto(url, { timeout: 5000, waitUntil: 'networkidle0' });

        await Promise.all([
            page.waitForNavigation(),
            page.click("button[name=reject]")
        ]);

        await sleep(2000);
    },
}
