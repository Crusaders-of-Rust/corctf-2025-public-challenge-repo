import { RedisClient, type BunRequest } from 'bun'
import type { FormData } from 'undici-types';
import puppeteer from 'puppeteer-extra';
import { Browser } from 'puppeteer'
import puppeteerExtraUserPrefs from 'puppeteer-extra-plugin-user-preferences';
import { randomBytes } from 'node:crypto';
import { writeFileSync } from 'node:fs'

puppeteer.use(puppeteerExtraUserPrefs({
	userPrefs: {
		net: {
			network_prediction_options: 2
		}
	}
}));

const sleep = time => new Promise(res => setTimeout(res, time));

const redis = new RedisClient('redis://redis:6379', { idleTimeout: 0, autoReconnect: true });

const headers = (type: string) => {
	return {
		headers: {
			'Content-Type': type,
			'Content-Security-Policy': [
				"default-src 'self' 'unsafe-inline'",
				"script-src 'none'"
			].join('; '),
			'X-Content-Type-Options': 'nosniff'
		}
	}
}

const routes = {
	'/': new Response(`
		<form action="/upload" method="POST" enctype="multipart/form-data">
			<input type="file" name="file">
			<input type="submit" value="upload">
		</form>
	`, headers('text/html')),

	'/upload': {
		POST: async (req: BunRequest): Promise<Response> => {
			let form: FormData;
			try {
				form = await req.formData();
			} catch(e) {
				return new Response('invalid upload!', headers('text/plain'));
			}
			const file = form.get('file');
			if (!file || !(file instanceof File) || !file.size || file.size > 2 ** 16) {
				return new Response('no file upload!', headers('text/plain'));
			}
			const id = await redis.incr('current-id');
			const data = JSON.stringify([file.type, (await file.bytes()).toBase64()]);
			await redis.set(`file|${id}`, data, 'EX', 10 * 60); // 10 minutes
			return Response.redirect(`/paper/${id}`);
		}
	},
	
	'/paper/:id': async (req: BunRequest<'/paper/:id'>): Promise<Response> => {
		const res = await redis.get(`file|${req.params.id}`);
		if (!res) {
			return new Response('not found!', headers('text/plain'));
		}
		const [type, data] = JSON.parse(res) as [string, string];
		return new Response(Buffer.from(data, 'base64'), headers(type));
	},

	'/secret': async (req: BunRequest): Promise<Response> => {
		const secret = req.cookies.get('secret') || '0123456789abcdef'.repeat(2);
		const payload = new URL(req.url, 'http://127.0.0.1').searchParams.get('payload') || '';

		return new Response(
			`<body secret="${secret}">${secret}\n${payload}</body>`,
			headers('text/html')
		);
	},

	'/flag': async (req: BunRequest): Promise<Response> => {
		const guess = new URL(req.url, 'http://127.0.0.1').searchParams.get('secret');
		const secret = await redis.getdel('secret');
		if (!secret) {
			return new Response('nice try', headers('text/plain'));
		}
		if (secret !== guess) {
			return new Response('wrong', headers('text/plain'));
		}
		return new Response(Bun.env.FLAG || 'corctf{flag}', headers('text/plain'));
	},

	'/visit/:id': async(req: BunRequest<'/visit/:id'>): Promise<Response> => {
		if (await redis.get('browser_open')) {
			return new Response('browser still open!');
		}

		const res = await redis.get(`file|${req.params.id}`);
		if (!res) {
			return new Response('not found!', headers('text/plain'));
		}
		const host = req.headers.get('Host') || '127.0.0.1:8080';
		let visit;

		// this is on prod :)
		if (Bun.env.VALIDATE) {
			if (!host || !host.match(/^paper-[a-f0-9]+\.ctfi\.ng$/)) {
				return new Response('invalid host header: ' + host, headers('text/plain'));
			}
			visit = `https://${host}/paper/${req.params.id}`;
		} else {
			visit = `http://127.0.0.1:8080/paper/${req.params.id}`;
		}

		writeFileSync('/etc/hosts', `178.156.204.114 ${host}`);

		const run = async () => {
			await redis.set('browser_open', 'true');

			const secret = randomBytes(16).toString('hex');

			let browser: Browser;

			try {
				browser = await puppeteer.launch({
					args: [
						'--no-sandbox',
						'--disable-gpu',
						'--js-flags=--noexpose_wasm,--jitless'
					],
					headless: true,
					pipe: true
				});
				await browser.setCookie({
					name: 'secret',
					value: secret,
					domain: host,
					sameSite: 'Strict'
				});
				await redis.set('secret', secret, 'EX', 60);
				const page = await browser.newPage();
				await page.goto(visit);
				await sleep(61000);
			} catch(e) {
				console.log(e);
			}

			try {
				if (browser) await browser.close();
			} catch(e) {};

			await redis.del('browser_open');
		}

		run();

		return new Response('visiting!', headers('text/plain'));
	}
}

Bun.serve({ port: 8080, maxRequestBodySize: 2 ** 16, development: false, routes });