// @ts-check
import { defineConfig } from "astro/config";
import { env } from "node:process";

const proxy_url = env.VSCODE_PROXY_URI;
const allowedHosts =
	proxy_url ?
		[
			proxy_url
				.replace("{{port}}", "4321")
				.replace(/^https:\/\//, "")
				.replace(/\/.*$/, ""),
		]
	:	[];

console.log(allowedHosts);

// https://astro.build/config
export default defineConfig({
	site: "https://blog.parrate.com",
	server: {
		allowedHosts,
	},
});
